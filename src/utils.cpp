#include "utils.h"

void clear_setlist() {
  for(auto e: setlist) {
    e.clear();
  }
  setlist.clear();
}

void first_config() {
    cfg.begin("config",false);

    cfg.putBool("init", true);
    #ifdef DEBUG
      Serial.print("first_cfg");
    #endif
    for(u_int8_t i=1; i<=AMT_PEDALS;i++) {
        cfg.putUChar(std::to_string(i).c_str(), 100+i);
    }
    cfg.end();
}

void load_config() {
    cfg.begin("config", true);
    for(u_int8_t i=1; i<=AMT_PEDALS;i++) {
        u_int8_t target = cfg.getUChar(std::to_string(i).c_str(), 0);
        if(target == 0XFF) {
          routings[i] = {OutputType::setlist_cmd, 0};
        } else {
        routings[i] = {OutputType::midi_cmd, target};
        }
    }
    cfg.end();
}

void send_config() {
  int index = 0;

  for(auto& it:routings) {
    bt_output_buffer[index] = it.first;
    if(it.second.type == OutputType::setlist_cmd) {
      bt_output_buffer[index+1] = 0xFF;
    } else {
      bt_output_buffer[index+1] = it.second.command;
    }
    index+=2;
  }
  SerialBT.write(bt_output_buffer, 2*AMT_PEDALS+1);
}

void update_config() {
  int index = 1;

  cfg.begin("config",false);

  u_int8_t pedal;
  u_int8_t value;

  while(true) {
    if(bt_input_buffer[index] == 0x00) {
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

void send_tempo_change() {
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

void send_midi_signal() {
  if(bt_input_buffer[1]==0xFF) {
    setlist_next();
  } else {
    sendOutput(bt_input_buffer[1]);
  }
  #ifdef DEBUG
    Serial.print("sending single midi signal");
  #endif
}

void pedal() {
  if(bt_input_buffer[1] == 0x00) {
    return;
  }
  sendOutput(routings[bt_input_buffer[1]].command);
  #ifdef DEBUG
    Serial.print("pressing pedal");
  #endif
}

void update_setlist() {
  clear_setlist();
  int idx = 1;

  std::vector<u_int8_t> item;

  while(true) {
    item.push_back(bt_input_buffer[idx]);
    idx++;

    if(bt_input_buffer[idx] == 0x00 && bt_input_buffer[idx+1] == 0x00) {
      setlist.push_back(item);
      break;
    }
    if(bt_input_buffer[idx] == 0x00) {
      setlist.push_back(item);
      item.clear();
      idx++;
    }
  }
}

void process_input() {
  u_int8_t first = bt_input_buffer[0];

  #ifdef DEBUG
    Serial.print("first: ");
    Serial.print(first);
    Serial.print("\n");
  #endif

  switch (first) {
    case 0:
      send_config();
      break;
    case 1:
      update_config();
      break;
    case 2:
      send_midi_signal();
      break;
    case 3:
      pedal();
      break;
    case 4:
      send_tempo_change();
      break;
    case 5:
      update_setlist();

  }
}

void BT_EventHandler(esp_spp_cb_event_t event, esp_spp_cb_param_t *param) {
  if (event == ESP_SPP_START_EVT) {
    #ifdef DEBUG
      Serial.println("Initialized SPP");
    #endif
  }
  else if (event == ESP_SPP_SRV_OPEN_EVT ) {
    set_LED(LED::BLUE);
    #ifdef DEBUG
      Serial.println("Client connected");
    #endif
  }
  else if (event == ESP_SPP_CLOSE_EVT  ) {
    #ifdef DEBUG
      Serial.println("Client disconnected");
    #endif
  }
  else if (event == ESP_SPP_DATA_IND_EVT ) {
    #ifdef DEBUG
      Serial.println("Data received");
    #endif
    int index = 0;
    while (SerialBT.available()) {
      int incoming = SerialBT.read();
      #ifdef DEBUG
        Serial.println(incoming);
      #endif
      bt_input_buffer[index] = incoming;
      index++;
    }
    bt_input_buffer[index] = 0x00;
    process_input();
  }
}

bool check_signal(u_int8_t pedal_nr, bool input) {
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

void set_LED(LED value) {
  switch (value) {
    case LED::GREEN:
      color = {0x00, 0xFF, 0x00};
      break;
    case LED::RED:
      color = {0xFF, 0x00, 0x00};
      break;
    case LED::BLUE:
      color = {0x00, 0x00, 0xFF};
      break;
  }
} 

void cycle_LED() {
  if(!color.empty()) {
    for(u_int16_t i = 0; i<LED_COUNT;i++) {
      leds.setPixelColor(i, brightness*color[0], brightness*color[1], brightness*color[2]);
    }

    if(brightness >= 0.98) {
      LED_down = true;
    } else if(brightness <= 0.02) {
      LED_down = false;
    }

    if(LED_down) {
      brightness -= BRIGHTNESS_STEP;
    } else {
      brightness += BRIGHTNESS_STEP;
    }
  }
}