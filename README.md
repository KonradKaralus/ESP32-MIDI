esp-midi


BT commands: 

*0x00 -> config-request
*0x01 -> new setup (end ist set by 0x00)
*0x02 -> send midi signal
*0x03 -> hit pedal
*0x04 -> change speed -> 1-4 f32 of tempo in LE


//0xFF in send_config and config store currently reserved for next_setlist