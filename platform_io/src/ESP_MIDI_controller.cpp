#include <utils.h>

ESP_MIDI_controller::ESP_MIDI_controller(
        void (*PC)(midi::DataByte PCnr, midi::Channel channel),
        void (*CC)(midi::DataByte CCnr, midi::DataByte CCvalue, midi::Channel channel)
        ): bluetooth(BT_Controller(this)) {
    this->CC = CC;
    this->PC = PC;
    LEDs = LED_Controller();
    cfg = Config_Controller();
    tempo_list_idx = 0;

    for(int i = 0; i<AMT_PEDALS; i++) {
        states[i] = pin_state{.state = false, .signal = 0};
    }
    pins[0] = 5;

    pinMode(5, INPUT_PULLDOWN);
    Serial.begin(115200);
    Serial2.begin(31250);
    pin_routings[5] = 1;
}

void ESP_MIDI_controller::pedal() {
    if(bt_input_buffer[1] == 0x00) {
        return;
    }
    sendOutput(cfg.routings[bt_input_buffer[1]].command);
    #ifdef DEBUG
        Serial.print("pressing pedal"); 
    #endif
}

void ESP_MIDI_controller::send_tempo(float tempo) {
    int u_delay = (60*1000000) / tempo;

    CC(midi::DataByte(0x40), 120, 1);
    delayMicroseconds(u_delay);
    CC(midi::DataByte(0x40), 120, 1);
    delay(200);
}

void ESP_MIDI_controller::send_tempo_change() {
    float f;
    uint8_t *f_ptr = (uint8_t *) &f;

    f_ptr[3] = bt_input_buffer[4];
    f_ptr[2] = bt_input_buffer[3];
    f_ptr[1] = bt_input_buffer[2];
    f_ptr[0] = bt_input_buffer[1];

    send_tempo(f);
    #ifdef DEBUG
        Serial.print("sending tempo change");
    #endif
}

void ESP_MIDI_controller::tempo_list_next() {
  #ifdef DEBUG
    Serial.println("tempo list next");
  #endif
  if(tempo_list_idx >= cfg.tempo_list.size()) {
    return;
  }
  tempo_list_idx++;  
  float tempo = cfg.tempo_list[tempo_list_idx];
  send_tempo(tempo);  
}

bool ESP_MIDI_controller::check_signal(u_int8_t pedal_nr, bool input) {
    pin_state* current = &states[pedal_nr - 1];

    if(input == current->state) {
    return false;
    }

    current->signal++;

    if(current->signal > TOLERANCE_CAP) {
    current->state = input;
    current->signal = 0;
    return true;
    }

    return false;
}


void ESP_MIDI_controller::sendOutput(u_int8_t msg) {
    uint8_t type = msg & 0x80;
    msg = msg & 0x7F;
    
    #ifdef DEBUG
      Serial.print(msg);
    #endif
    if(type == 0) {
        PC(msg, 1);
    } else {
        CC(midi::DataByte(msg), CC_DEFAULT, 1);
    }
    delay(200);
}

void ESP_MIDI_controller::loop() {
  u_int8_t pedal_nr;

  for(u_int8_t pin_nr : pins) {
    pedal_nr = pin_routings[pin_nr];

    if(check_signal(pedal_nr, (bool)digitalRead(pin_nr))) {
      if(cfg.routings[pedal_nr].type == OutputType::midi_cmd) {
        sendOutput(cfg.routings[pedal_nr].command);
      } else if(cfg.routings[pedal_nr].type == OutputType::tempo_list_cmd) {
        tempo_list_next();
      }
    }
  } 

    LEDs.cycle_LED();
    if(cfg.cfg_updated) {
      cfg.load_config();
      cfg.cfg_updated = false;

      #ifdef DEBUG
        Serial.print("new cfg applied");
      #endif
    }
}
