esp-midi


BT commands: 

*0x00 -> config-request
*0x01 -> new setup (end ist set by 0x00)
*0x02 -> send midi signal
*0x03 -> hit pedal
*0x04 -> change speed -> 1-4 f32 of tempo in LE
*0x05 -> new tempolist: 4 bytes each tempo f32, double 0x00 is end, max size: 32 tempos -> 4*32 + 3 = 131 buffer  
128,PC2,CC7|0,CC1,PC7|... first entry is always tempo, 1->default


//0xFF in send_config and config store currently reserved for next_setlist
