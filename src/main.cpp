#include "Midi.h"
#include "Preferences.h"
#include "string"
#include "unordered_map"
#include "BluetoothSerial.h"

#define AMT_PEDALS 5
#define CC_DEFAULT 0
#define DO_FIRST_CFG true

MIDI_CREATE_INSTANCE(HardwareSerial, Serial2, DIN_MIDI);

std::unordered_map<u_int8_t, uint8_t> routings;
Preferences cfg;
midi::Channel channel(0x0);
BluetoothSerial SerialBT;

bool states[AMT_PEDALS];

u_int8_t pins[] = {5};
std::unordered_map<u_int8_t, uint8_t> pin_routings;

u_int8_t bt_input_buffer[2*AMT_PEDALS + 1];
u_int8_t bt_output_buffer[2*AMT_PEDALS + 1];


bool cfg_updated = false;

void first_config() {
    cfg.begin("config",false);

    cfg.putBool("init", true);
    Serial.print("first_cfg");
    for(u_int8_t i=1; i<=AMT_PEDALS;i++) {
        cfg.putUChar(std::to_string(i).c_str(), 120+i);
    }
    cfg.end();
}

void load_config() {
    cfg.begin("config", true);
    for(u_int8_t i=1; i<=AMT_PEDALS;i++) {
        u_int8_t target = cfg.getUChar(std::to_string(i).c_str(), 0);
        routings[i] = target;
    }
    cfg.end();
}

void send_config() {


  int index = 0;

  for(auto& it:routings) {
    bt_output_buffer[index] = it.first;
    bt_output_buffer[index+1] = it.second;
    index+=2;
  }

  SerialBT.write(bt_output_buffer, 2*AMT_PEDALS+1);
}

//first: 0x00 -> request setup, first: 0xFF -> setup change

void sendOutput(u_int8_t msg) {

    uint8_t type = msg & 0x80;

    msg = msg & 0x7F;

    Serial.print(msg);

    if(type == 0) {
        DIN_MIDI.sendProgramChange(midi::DataByte(msg), channel);
    } else {
        DIN_MIDI.sendControlChange(midi::DataByte(msg), CC_DEFAULT, channel);
    }
}

void process_input() {
  u_int8_t first = bt_input_buffer[0];

  if(first == 0x00) {
    send_config();
    return;
  }

  int index = 1;

  cfg.begin("config",false);

  u_int8_t pedal;
  u_int8_t value;

  while(true) {
    if(bt_input_buffer[index] == 0x00) { //0x00 as first in sequence -> break; -> Pedal no. 0 cannot exist
      break;
    } 
    pedal = bt_input_buffer[index];
    value = bt_input_buffer[index+1];

    cfg.putUChar(std::to_string(pedal).c_str(), value);

    index += 2;
  }
  cfg.end();

  cfg_updated = true;
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
    int index = 0;
    while (SerialBT.available()) {
      int incoming = SerialBT.read();
      Serial.println(incoming);
      bt_input_buffer[index] = incoming;
      index++;
    }
    bt_input_buffer[index] = 0x00;
    process_input();
  }
}




void setup() {
    Serial.begin(115200);
    DIN_MIDI.begin(MIDI_CHANNEL_OMNI);

    cfg.begin("config",true);

    bool init = cfg.isKey("init");

    if(!init || DO_FIRST_CFG) {
        cfg.end();
        first_config();
    }

    load_config();

    pinMode(5, INPUT_PULLDOWN);
    
    pin_routings[5] = 5;

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

  u_int8_t pedal_nr;

  for(u_int8_t pin_nr : pins) {
    pedal_nr = pin_routings[pin_nr];
    if(digitalRead(pin_nr)) {

      if(!states[pedal_nr-1]) {
        states[pedal_nr-1] = true;
        sendOutput(routings[pedal_nr]);
      }
    } else {
      states[pedal_nr-1] = false;
    }
  }
    // esp_deep_sleep(200000);

    if(cfg_updated) {
      load_config();
      cfg_updated = false;
      Serial.print("new cfg applied");
    }
}