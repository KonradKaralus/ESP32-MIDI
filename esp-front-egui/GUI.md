## Bluetooth commands:
First byte defines the type of the command
```
- 0x00 -> request of current configuration
- 0x01 -> new configuration (end ist set by 0x00)
- 0x02 -> send single midi signal
- 0x03 -> hit pedal No.
- 0x04 -> change speed (tempo is 4 byte float in LE) 
- 0x05 -> new tempolist: 4 bytes for each tempo (end is set by 0x00)  
```
Due to the dynamic length of the tempolist command, the current length of the Bluetooth buffer on the ESP32 is 134 bytes (4\*32 + 6), which allows for a maximum of 32 tempos in the tempo list  

>[!IMPORTANT]
> 0xFF is currently reserved for the command, that iterates the tempolist, thus PC127 is not a valid command
