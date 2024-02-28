#include "utils.h"

void first_config() {
    cfg.begin("config",false);

    cfg.putBool("init", true);
    #ifdef DEBUG
      Serial.print("first_cfg");
    #endif
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

void update_config() {
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

void send_midi_signal() {
  sendOutput(bt_input_buffer[1]);
  #ifdef DEBUG
    Serial.print("sending single midi signal");
  #endif
}

void pedal() {
  if(bt_input_buffer[1] == 0x00) {
    return;
  }
  sendOutput(routings[bt_input_buffer[1]]);
  #ifdef DEBUG
    Serial.print("pressing pedal");
  #endif
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
  }
}

void BT_EventHandler(esp_spp_cb_event_t event, esp_spp_cb_param_t *param) {
  if (event == ESP_SPP_START_EVT) {
    #ifdef DEBUG
      Serial.println("Initialized SPP");
    #endif
  }
  else if (event == ESP_SPP_SRV_OPEN_EVT ) {
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