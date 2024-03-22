#include "MIDI.h"
#include "Preferences.h"
#include "string"
#include "unordered_map"
#include "BluetoothSerial.h"
#include "vector"
#include <Adafruit_NeoPixel.h>

#define AMT_PEDALS 6
#define CC_DEFAULT 0
#define DEBUG true
#define TOLERANCE_CAP 500
#define PIN 4
#define LED_COUNT 4
#define BRIGHTNESS_STEP 0.001

extern Adafruit_NeoPixel leds;

struct pin_state {
  bool state;
  int signal;
};

enum OutputType { midi_cmd, tempo_list_cmd };
enum LED { RED, GREEN, BLUE };

extern std::array<u_int8_t, 3> color;
extern float brightness; //clamped 0->1
extern bool LED_down;

struct output {
  OutputType type;
  u_int8_t command;
};

extern std::unordered_map<u_int8_t, output> routings;
extern std::vector<float> tempo_list;

extern Preferences cfg;
extern BluetoothSerial SerialBT;

extern pin_state states[AMT_PEDALS];

extern u_int8_t pins[];
extern std::unordered_map<u_int8_t, u_int8_t> pin_routings;

extern u_int8_t bt_input_buffer[131];
extern u_int8_t bt_output_buffer[131];

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
void send_tempo_change();
void update_tempo_list();
void clear_tempo_list();
void tempo_list_next();

void set_LED(LED value);

void cycle_LED();

class ESP_MIDI_controller;

class Config_Controller {
  public:
    Config_Controller();
    std::unordered_map<u_int8_t, output> routings;
    std::vector<float> tempo_list;
    void load_config();
    void update_config();
    void update_tempo_list();
    bool cfg_updated;
  private:
    void first_config();
    void clear_tempo_list();
    Preferences cfg;
};

class BT_Controller {
  public:
    BT_Controller();
    BT_Controller(ESP_MIDI_controller* parent);
    void send_config(Config_Controller* cfg);
    void BT_EventHandler(esp_spp_cb_event_t event, esp_spp_cb_param_t *param);
    void process_input();
    bool cfg_updated;
  private:
    ESP_MIDI_controller* midi_controller;
    BluetoothSerial SerialBT;
    // u_int8_t bt_input_buffer[131];
    // u_int8_t bt_output_buffer[131];
};

class LED_Controller {
  public:
    LED_Controller();
    void set_LED(LED value);
    void cycle_LED();
  private:
    Adafruit_NeoPixel leds;
    std::array<u_int8_t, 3> color;
    float brightness; //clamped 0->1
    bool LED_down;
};

class ESP_MIDI_controller {
  public:
    ESP_MIDI_controller(
      void (*PC)(midi::DataByte PCnr, midi::Channel channel),
      void (*CC)(midi::DataByte CCnr, midi::DataByte CCvalue, midi::Channel channel)
      );
    void pedal();
    void send_tempo(float tempo);
    void send_tempo_change();
    void tempo_list_next();
    void (*PC)(midi::DataByte PCnr, midi::Channel channel);
    void (*CC)(midi::DataByte CCnr, midi::DataByte CCvalue, midi::Channel channel);
    void loop();
    void sendOutput(u_int8_t msg);
    bool check_signal(u_int8_t pedal_nr, bool input);
  private:
    Config_Controller cfg;
    LED_Controller LEDs;
    BT_Controller bluetooth;
    pin_state states[AMT_PEDALS];
    u_int8_t pins[1];
    std::unordered_map<u_int8_t, u_int8_t> pin_routings;
    unsigned int tempo_list_idx;
};