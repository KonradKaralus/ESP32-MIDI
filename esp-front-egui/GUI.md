>[!IMPORTANT]
> Be sure to enter your ESP's BT address in `main.rs`

## Command Syntax
- Footswitch and single MIDI command: PCvalue or CCvalue, i.e. PC120 or CC1
- Single pedal press: Pedal number, i.e. 5
- Single tempo change: tempo (decimal), i.e. 80.5
- Tempo list: list of single tempos, separated by ","  i.e. 120.1,90,100.123

## Supported platforms
The GUI application only supports Windows

## Bluetooth commands:
First byte defines the type of the command
```
- 0x00 -> request of current configuration
- 0x01 -> new configuration (end ist set by 0x00)
- 0x02 -> heartbeat request
- 0x04 -> change speed (tempo is 4 byte float in LE) 
```

>[!IMPORTANT]
> Note that Preset-Up (up) and Preset-Down (down) will only work if you set your HX-Device to use FS-4 and FS-5 to act as Preset-Up and Preset-Down respectively.
