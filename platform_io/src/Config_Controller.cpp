#include <utils.h>

Config_Controller::Config_Controller() {
    cfg.begin("config",true);
    bool init = cfg.isKey("init");
    if(!init || DEBUG) {
        cfg.end();
        first_config();
    }

    load_config();
    cfg_updated = false;
}



void Config_Controller::load_config() {
    cfg.begin("config", true);
    for(u_int8_t i=1; i<=AMT_PEDALS;i++) {
      u_int8_t target = cfg.getUChar(std::to_string(i).c_str(), 0);
      if(target == 0XFF) {
        routings[i] = {OutputType::tempo_list_cmd, 0};
      } else {
      routings[i] = {OutputType::midi_cmd, target};
      }
    }
    
    tempo_list.clear();
    for(int i = 0; i<int(cfg.getUChar("tempo_size")); i++) {
      float tempo = cfg.getFloat(("T"+std::to_string(i)).c_str());
      tempo_list.push_back(tempo);
    }
    cfg.end();
};
void Config_Controller::update_config() {
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
};

void Config_Controller::update_tempo_list() {
    cfg.begin("config",false);
    int tempolist_idx = 0;
    int idx = 1;
    while(true) {
        float f;
        uint8_t *f_ptr = (uint8_t *) &f;

        f_ptr[3] = bt_input_buffer[idx+3];
        f_ptr[2] = bt_input_buffer[idx+2];
        f_ptr[1] = bt_input_buffer[idx+1];
        f_ptr[0] = bt_input_buffer[idx];

        cfg.putUChar(("T"+std::to_string(tempolist_idx)).c_str(), f);
        idx+=4;
        if(bt_input_buffer[idx] == 0x00 && bt_input_buffer[idx+1] == 0x00) {
            break;
        }
        tempolist_idx++;
    }

    cfg.putUChar("tempo_size", tempolist_idx+1);

    cfg.end();
    cfg_updated = true;
};
void Config_Controller::first_config() {
    cfg.begin("config",false);

    cfg.putBool("init", true);
    #ifdef DEBUG
      Serial.print("first_cfg");
    #endif
    for(u_int8_t i=1; i<=AMT_PEDALS;i++) {
        cfg.putUChar(std::to_string(i).c_str(), 100+i);
    }
    cfg.putUChar("tempo_size", 0);

    cfg.end();
};

void Config_Controller::clear_tempo_list() {
    tempo_list.clear();
};