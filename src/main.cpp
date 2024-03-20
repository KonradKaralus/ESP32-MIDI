#include "MIDI.h"
#include "Preferences.h"
#include "string"
#include "unordered_map"
#include "vector"
#include "BluetoothSerial.h"
#include "utils.h"

MIDI_CREATE_INSTANCE(HardwareSerial, Serial2, MIDI);

Adafruit_NeoPixel leds = Adafruit_NeoPixel(LED_COUNT, PIN, NEO_GRB + NEO_KHZ800);

std::vector<std::vector<u_int8_t>> setlist;

std::unordered_map<u_int8_t, output> routings; // command-routing
Preferences cfg;
BluetoothSerial SerialBT;

pin_state states[AMT_PEDALS];

u_int8_t pins[] = {5};
std::unordered_map<u_int8_t, uint8_t> pin_routings; // hardware-routing

u_int8_t bt_input_buffer[545];
u_int8_t bt_output_buffer[545];

bool cfg_updated = false;

unsigned int setlist_idx = 0;
float brightness = 0;

std::array<u_int8_t, 3> color;

bool LED_down = false;

void sendOutput(u_int8_t msg) {

    uint8_t type = msg & 0x80;

    msg = msg & 0x7F;
    
    #ifdef DEBUG
      Serial.print(msg);
    #endif

    if(type == 0) {
        MIDI.sendProgramChange(msg, 1);
    } else {
        MIDI.sendControlChange(midi::DataByte(msg), CC_DEFAULT, 1);
    }
    delay(200);
}

void send_tempo(float tempo) {
  int u_delay = (60*1000000) / tempo;

  MIDI.sendControlChange(midi::DataByte(0x40), 120, 1);
  delayMicroseconds(u_delay);
  MIDI.sendControlChange(midi::DataByte(0x40), 120, 1);
  delay(200);
}

void setlist_next() {
  #ifdef DEBUG
    Serial.println("setlist next");
  #endif
  if(setlist_idx >= setlist.size()) {
    return;
  }
  auto commands = setlist[setlist_idx];

  if(commands[0] != 0x01) {
    send_tempo(commands[0]);
  }

  for(int i=1;i<commands.size();i++) {
    sendOutput(commands[i]);
  }
  
  setlist_idx++;  
}


void setup() {
  MIDI.begin(1);
  leds.begin();
  // Serial2.begin(115200);
  pinMode(5, INPUT_PULLDOWN);
  
  Serial.begin(115200);

  for(int i = 0; i<AMT_PEDALS; i++) {
    pin_state ps;
    ps.signal = 0;
    ps.state = false;
    states[i] = ps;
  }

  Serial2.begin(31250);

  cfg.begin("config",true);

  bool init = cfg.isKey("init");

  if(!init || DEBUG) {
      cfg.end();
      first_config();
  }

  load_config();

  pinMode(5, INPUT_PULLDOWN);
  
  pin_routings[5] = 1;

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

    if(check_signal(pedal_nr, (bool)digitalRead(pin_nr))) {
      Serial.print("type:"); Serial.println(routings[pedal_nr].type);

      if(routings[pedal_nr].type == OutputType::midi_cmd) {
        sendOutput(routings[pedal_nr].command);
      } else if(routings[pedal_nr].type == OutputType::setlist_cmd) {
        setlist_next();
      }
    }
  } 

    cycle_LED();
    if(cfg_updated) {
      load_config();
      cfg_updated = false;

      #ifdef DEBUG
        Serial.print("new cfg applied");
        Serial.print("cfg:"); Serial.println(routings[1].type);

      #endif
    }
}