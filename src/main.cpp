#include "Midi.h"
#include "Preferences.h"
#include "string"
#include "unordered_map"
#include "BluetoothSerial.h"
#include "mutex"

#define AMT_PEDALS 5
#define CC_DEFAULT 0

MIDI_CREATE_INSTANCE(HardwareSerial, Serial2, DIN_MIDI);

std::unordered_map<u_int8_t, uint8_t> routings;
Preferences cfg;
midi::Channel channel(0x0);
BluetoothSerial SerialBT;

bool cfg_updated = false;
std::mutex cfg_mutex;

void first_config() {
    cfg.begin("config",false);

    cfg.putBool("init", true);

    for(u_int8_t i=0; i<AMT_PEDALS;i++) {
        cfg.putUChar(std::to_string(i).c_str(), 0);
    }

    cfg.end();
}

void load_config() {
    cfg.begin("config", true);
    for(u_int8_t i=0; i<AMT_PEDALS;i++) {
        u_int8_t target = cfg.getUChar(std::to_string(i).c_str(), 0);
        routings[i] = target;
    }
    cfg.end();
}

void sendOutput(u_int8_t msg) {

    uint8_t type = msg & 0x80;

    msg = msg & 0x7F;

    if(type == 0) {
        DIN_MIDI.sendProgramChange(midi::DataByte(msg), channel);
    } else {
        DIN_MIDI.sendControlChange(midi::DataByte(msg), CC_DEFAULT, channel);
    }
}

void BT_EventHandler(esp_spp_cb_event_t event, esp_spp_cb_param_t *param) {
  if (event == ESP_SPP_START_EVT) {
    Serial.println("Initialized SPP");
  }
  else if (event == ESP_SPP_SRV_OPEN_EVT ) {
    Serial.println("Client connected");
  }
  else if (event == ESP_SPP_CLOSE_EVT  ) {
    Serial.println("Client disconnected");
  }
  else if (event == ESP_SPP_DATA_IND_EVT ) {
    Serial.println("Data received");
    while (SerialBT.available()) {
      int incoming = SerialBT.read();
      Serial.println(incoming);
    }
  }
}

void setup() {
    Serial.begin(115200);
    DIN_MIDI.begin(MIDI_CHANNEL_OMNI);

    cfg.begin("config",true);

    bool init = cfg.isKey("init");

    if(!init) {
        cfg.end();
        first_config();
    }

    load_config();

    

    // if (esp_base_mac_addr_set(newMAC) == ESP_OK) {
    //     Serial.println("MAC address set successfully");
    // } else {
    //     Serial.println("Failed to set MAC address");
    // }

    SerialBT.begin("ESP");
    SerialBT.setPin("1");
    SerialBT.register_callback(BT_EventHandler);


    
}


/*
self doc for u_int8 in routings/cfg
first bit indicates if PC or CC
0 -> PC
1 -> CC

last seven bits are message
*/
void loop() {

    // for(u_int8_t i; i<AMT_PEDALS;i++) {
    //     if(digitalRead(i)) {
    //         sendOutput(routings[i]);
    //     }
    // }

    // esp_deep_sleep(200000);
}