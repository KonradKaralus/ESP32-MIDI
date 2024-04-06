#include "MIDI.h"
#include "Preferences.h"
#include "string"
#include "unordered_map"
#include "vector"
#include "BluetoothSerial.h"
#include "utils.h"


MIDI_CREATE_INSTANCE(HardwareSerial, Serial2, MIDI);

Adafruit_NeoPixel leds = Adafruit_NeoPixel(LED_COUNT, PIN, NEO_GRB + NEO_KHZ800);

std::vector<float> tempo_list;
pthread_t tempo_thread;

std::unordered_map<u_int8_t, output> routings; // command-routing
Preferences cfg;
BluetoothSerial SerialBT;

pin_state states[AMT_PEDALS];

u_int8_t pins[] = {13,14,27,26,25,33};
std::unordered_map<u_int8_t, uint8_t> pin_routings; // hardware-routing

u_int8_t bt_input_buffer[134];
u_int8_t bt_output_buffer[134];

bool cfg_updated = false;

unsigned int tempo_list_idx = 0;
double brightness = 0.8;

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
        Serial.println("PC");
    } else {
        MIDI.sendControlChange(midi::DataByte(msg), CC_DEFAULT, 1);
    }
    delay(200);
}

void *thread_tempo(void *tempo) {
  int u_delay = (60*1000000) / *((float*)tempo);

  MIDI.sendControlChange(midi::DataByte(0x40), 120, 1);
  delayMicroseconds(u_delay);
  MIDI.sendControlChange(midi::DataByte(0x40), 120, 1);
  delay(200);

  pthread_exit(NULL);
}

void send_tempo(float tempo) {
  int ret = pthread_create(&tempo_thread, NULL, thread_tempo, (void *) &tempo);
  if (ret) {
      Serial.println("An error has occurred");
  }
}

void tempo_list_next() {
  #ifdef DEBUG
    Serial.println("tempo list next");
    Serial.println(tempo_list_idx+1);
    Serial.println(tempo_list.size());
  #endif
  if(tempo_list_idx >= tempo_list.size()) {
    return;
  }
  tempo_list_idx++;  
  float tempo = tempo_list[tempo_list_idx];
  send_tempo(tempo);  
}

//currently not in use
void tempo_list_prev() {
  #ifdef DEBUG
    Serial.println("tempo list next");
  #endif
  if(tempo_list_idx <= tempo_list.size()) {
    return;
  }
  tempo_list_idx--;  

  float tempo = tempo_list[tempo_list_idx];
  send_tempo(tempo);  
}


void setup() {
  MIDI.begin(1); //todo use!!!

  pinMode(13, INPUT_PULLUP);
  pinMode(14, INPUT_PULLUP);
  pinMode(27, INPUT_PULLUP);
  pinMode(26, INPUT_PULLUP);
  pinMode(25, INPUT_PULLUP);
  pinMode(33, INPUT_PULLUP);
  
  Serial.begin(115200);

  Serial2.begin(31250);

  cfg.begin("config",true);

  bool init = cfg.isKey("init");

  if(!init || DEBUG) {
      cfg.end();
      first_config();
  }

  load_config();
  
  pin_routings[13] = 1;
  pin_routings[14] = 2;
  pin_routings[27] = 3;
  pin_routings[26] = 4;
  pin_routings[25] = 5;
  pin_routings[33] = 6;

  for(int i = 0; i<AMT_PEDALS; i++) {
    pin_state ps;
    ps.signal = 0;
    // ps.state = false;
    ps.state = (bool)digitalRead(pin_routings[pins[i]]);
    states[i] = ps;
  }

  SerialBT.begin("ESP");
  SerialBT.setPin("1");
  SerialBT.register_callback(BT_EventHandler);

  set_LED(LED::GREEN);
  leds.begin();
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
      set_LED(LED::GREEN);


      if(routings[pedal_nr].type == OutputType::midi_cmd) {
        sendOutput(routings[pedal_nr].command);
      } else if(routings[pedal_nr].type == OutputType::tempo_list_cmd) {
        tempo_list_next();
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