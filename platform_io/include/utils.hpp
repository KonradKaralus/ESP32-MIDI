#include "MIDI.h"
#include "Preferences.h"
#include "string"
#include "unordered_map"
#include "BluetoothSerial.h"
#include "vector"
#include "Adafruit_NeoPixel.h"
#include "pthread.h"

#define AMT_PEDALS 6
#define CC_DEFAULT 0
#define TOLERANCE_CAP 100
#define PIN 4
#define LED_COUNT 4
#define BRIGHTNESS_STEP 0.0002

extern Adafruit_NeoPixel leds;

struct pin_state
{
  bool state;
  int signal;
};

struct Command
{
  /// 0: PC, 255: CC
  u_int8_t signal_type;
  u_int8_t value;
  u_int8_t on_activate;
  u_int8_t on_deactivate;
  u_int8_t channel;
};
enum LED
{
  RED,
  GREEN,
  BLUE
};

extern std::array<u_int8_t, 3> color;
extern double brightness; // clamped 0->1
extern bool LED_down;

extern std::unordered_map<u_int8_t, Command> routings;

extern pthread_t tempo_thread;

extern Preferences cfg;
extern BluetoothSerial SerialBT;

extern pin_state states[AMT_PEDALS];

extern u_int8_t pins[];
extern std::unordered_map<u_int8_t, u_int8_t> pin_routings;

extern u_int8_t bt_input_buffer[134];
extern u_int8_t bt_output_buffer[134];

extern bool cfg_updated;

void first_config();
void load_config();
void send_config();
void sendOutput(u_int8_t msg);
void update_config();
void send_midi_signal();
void process_input();
void BT_EventHandler(esp_spp_cb_event_t event, esp_spp_cb_param_t *param);
bool check_signal(u_int8_t pedal_nr, bool input);
void pedal();
void send_tempo(float tempo);
void *thread_tempo(void *tempo);
void send_tempo_change();

void set_LED(LED value);
void cycle_LED();