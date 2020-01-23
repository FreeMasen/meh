# Meh-rs

This is a binary application for consuming the [Meh.com API](https://meh.com/forum/topics/meh-api).

## Usage
```bash
meh 
A cli tool for watching meh.com deals

USAGE:
    meh.exe [FLAGS] [OPTIONS]

FLAGS:
    -h, --help        
            Prints help information

    -p, --progress    
            If the cli should constantly report the current deal and when the next check will happen This is useful for
            active monitoring, if not passed you may want to pass the alert otherwise the application will not have any
            way to communicate
    -V, --version     
            Prints version information


OPTIONS:
    -a, --alert <FILE>    
            Path to file that should be executed when the deal changes On windows this will be executed through
            powershell, on unix like it will executed directly
```

When the `--progress` flag is provided, the following will be printed to the screen and
updated periodically.
```
▒ Bag of Crap (5.00)
▒ https://meh.com/deals/bag-of-crap
▒ checking again in 12 hours at 10:00 pm
```

When the `--alert` argument is provided, the program will attempt to execute the provided file (powershell on windows and sh on unix) when the deal changes.

Regardless of the operating system, a single argument will be passed that will include the current deal's title and the dolar amounts (if multiple price points all will be included).

### Alert Example
#### Windows
```powershell
# ./alert.ps1
param (
    [string]$MSG = ""
)
Add-Type -AssemblyName System.Windows.Forms
$global:balmsg = New-Object System.Windows.Forms.NotifyIcon
$path = (Get-Process -id $pid).Path
$balmsg.Icon = [System.Drawing.Icon]::ExtractAssociatedIcon($path)
$balmsg.BalloonTipIcon = [System.Windows.Forms.ToolTipIcon]::Warning
$balmsg.BalloonTipText = $MSG
$balmsg.BalloonTipTitle = "New MEH Deal"
$balmsg.Visible = $true
Remove-Event notification_event -ea SilentlyContinue
Register-ObjectEvent $balmsg BalloonTipClicked -Action {
    Start-Process "https://meh.com"
}

$balmsg.ShowBalloonTip(20000)
Wait-Event -Timeout 20

```
This powershell script will create an alert bubble, which when clicked will
open the user's default browser to meh.com. The content of the alert is
provided by the $MSG argument

You could pass this as the alert argument to `meh.exe` and it will provide
a desktop alert when the deal changes. like so:
```powershell
meh.exe -a ./alert.ps1
```
Which might look like this:
![windows alert box](assets/meh_alert.jpg)

#### Unix
This example assumes an Ubuntu system with `notify-send` installed, 

```bash
# ./alert.sh
notify-send "New Meh.com deal" "$1"
```
The above should display an alert that might look something like this:

![Ubuntu alert box](assets/ubuntu_alert.png)