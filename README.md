esp-midi


BT commands: 

*0x00 -> config-request
*0x01 -> new setup (end ist set by 0x00)
*0x02 -> send midi signal
*0x03 -> hit pedal
*0x04 -> change speed -> 1-4 f32 of tempo in LE
*0x05 -> new setlist: 0x00 marks border between stages, max size for: 32 Stages *  16 cmds + 1 start + 32 borders = 545 entries  T128,PC2,CC7|CC1,PC7|...


//0xFF in send_config and config store currently reserved for next_setlist