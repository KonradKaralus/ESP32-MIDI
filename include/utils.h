#include "Midi.h"
#include "Preferences.h"
#include "string"
#include "unordered_map"
#include "BluetoothSerial.h"

#define AMT_PEDALS 6
#define CC_DEFAULT 0
#define DEBUG true
#define TOLERANCE_CAP 500

struct pin_state {
  bool state;
  int signal;
};

extern std::unordered_map<u_int8_t, uint8_t> routings;
extern Preferences cfg;
extern BluetoothSerial SerialBT;

extern pin_state states[AMT_PEDALS];

extern u_int8_t pins[];
extern std::unordered_map<u_int8_t, uint8_t> pin_routings;

extern u_int8_t bt_input_buffer[2*AMT_PEDALS + 1];
extern u_int8_t bt_output_buffer[2*AMT_PEDALS + 1];

extern bool cfg_updated;

void first_config();
void load_config();
void send_config();
void sendOutput(u_int8_t msg);
void update_config();
void send_midi_signal();
void pedal();
void process_input();
void BT_EventHandler(esp_spp_cb_event_t event, esp_spp_cb_param_t *param);
bool check_signal(u_int8_t pedal_nr, bool input);
void send_tempo(u_int tempo);
void send_tempo_change();