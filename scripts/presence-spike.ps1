<#
.SYNOPSIS
  Build step 1 presence test: does an injected mouse move reset Windows idle time?

.EXAMPLE
  ./scripts/presence-spike.ps1 -Style Zero -Minutes 20
  ./scripts/presence-spike.ps1 -Style Pair -Pixels 1 -Minutes 20
#>
param(
    [ValidateSet('Zero', 'Pair', 'None')] [string] $Style = 'Zero',
    [ValidateRange(1, 5)] [int] $Pixels = 1,
    [int] $Minutes = 20,
    [int] $IntervalSeconds = 60
)

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

public static class Spike {
    [StructLayout(LayoutKind.Sequential)]
    struct MOUSEINPUT { public int dx, dy; public uint mouseData, dwFlags, time; public IntPtr dwExtraInfo; }
    [StructLayout(LayoutKind.Sequential)]
    struct INPUT { public uint type; public MOUSEINPUT mi; }
    [StructLayout(LayoutKind.Sequential)]
    struct LASTINPUTINFO { public uint cbSize, dwTime; }

    [DllImport("user32.dll", SetLastError = true)] static extern uint SendInput(uint n, INPUT[] inputs, int size);
    [DllImport("user32.dll")] static extern bool GetLastInputInfo(ref LASTINPUTINFO lii);
    [DllImport("kernel32.dll")] static extern uint GetTickCount();
    [DllImport("user32.dll")] public static extern bool GetCursorPos(out POINT p);
    [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }

    const uint MOUSEEVENTF_MOVE = 0x0001;

    public static uint IdleMs() {
        var lii = new LASTINPUTINFO { cbSize = (uint)Marshal.SizeOf(typeof(LASTINPUTINFO)) };
        GetLastInputInfo(ref lii);
        return unchecked(GetTickCount() - lii.dwTime);
    }

    static INPUT Move(int dx) {
        return new INPUT { type = 0, mi = new MOUSEINPUT { dx = dx, dwFlags = MOUSEEVENTF_MOVE } };
    }

    public static uint Inject(string style, int px) {
        INPUT[] inputs = style == "Pair" ? new[] { Move(px), Move(-px) } : new[] { Move(0) };
        return SendInput((uint)inputs.Length, inputs, Marshal.SizeOf(typeof(INPUT)));
    }
}
'@

Write-Host "Style=$Style Pixels=$Pixels Minutes=$Minutes Interval=${IntervalSeconds}s"
Write-Host "time      idle_before_ms  sent  idle_after_ms  cursor_before  cursor_after"

$end = (Get-Date).AddMinutes($Minutes)
while ((Get-Date) -lt $end) {
    $before = [Spike]::IdleMs()
    $p0 = New-Object Spike+POINT; [void][Spike]::GetCursorPos([ref]$p0)
    $sent = if ($Style -eq 'None') { 0 } else { [Spike]::Inject($Style, $Pixels) }
    Start-Sleep -Milliseconds 50
    $after = [Spike]::IdleMs()
    $p1 = New-Object Spike+POINT; [void][Spike]::GetCursorPos([ref]$p1)
    '{0:HH:mm:ss}  {1,14}  {2,4}  {3,13}  {4,13}  {5,12}' -f (Get-Date), $before, $sent, $after, "$($p0.X),$($p0.Y)", "$($p1.X),$($p1.Y)"
    Start-Sleep -Seconds $IntervalSeconds
}
