#include "utils.hpp"

MIDI_CREATE_INSTANCE(HardwareSerial, Serial2, MIDI);

Adafruit_NeoPixel leds = Adafruit_NeoPixel(LED_COUNT, PIN, NEO_GRB + NEO_KHZ800);

pthread_t tempo_thread;

std::unordered_map<u_int8_t, Command> routings; // command-routing
Preferences cfg;
BluetoothSerial SerialBT;

pin_state states[AMT_PEDALS];

u_int8_t pins[] = {13, 14, 27, 26, 25, 33};
std::unordered_map<u_int8_t, uint8_t> pin_routings; // hardware-routing

u_int8_t bt_input_buffer[134];
u_int8_t bt_output_buffer[134];

bool cfg_updated = false;

double brightness = 0.8;

std::array<u_int8_t, 3> color;

bool LED_down = false;

void sendOutput(Command *cmd, bool state)
{
  switch (cmd->signal_type)
  {
  case 0x00:
    MIDI.sendProgramChange(cmd->value, cmd->channel);
    break;
  case 0xff:
    MIDI.sendControlChange(cmd->value, state ? cmd->on_activate : cmd->on_deactivate, cmd->channel);
    break;

  default:
    break;
  }
  delay(200);
}

/*
void *thread_tempo(void *tempo)
{
  int u_delay = (60 * 1000000) / *((float *)tempo);

  MIDI.sendControlChange(midi::DataByte(0x40), 120, 1);
  delayMicroseconds(u_delay);
  MIDI.sendControlChange(midi::DataByte(0x40), 120, 1);
  delay(200);

  pthread_exit(NULL);
}

void send_tempo(float tempo)
{
  int ret = pthread_create(&tempo_thread, NULL, thread_tempo, (void *)&tempo);
  if (ret)
  {
    Serial.println("An error has occurred");
  }
}
*/
void setup()
{
  MIDI.begin(1); // todo use!!!

  pinMode(13, INPUT_PULLUP);
  pinMode(14, INPUT_PULLUP);
  pinMode(27, INPUT_PULLUP);
  pinMode(26, INPUT_PULLUP);
  pinMode(25, INPUT_PULLUP);
  pinMode(33, INPUT_PULLUP);

  Serial.begin(115200);

  Serial2.begin(31250);

  cfg.begin("config", true);

  bool init = cfg.isKey("init2");

  if (!init)
  {
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

  for (int i = 0; i < AMT_PEDALS; i++)
  {
    pin_state ps;
    ps.signal = 0;
    ps.state = (bool)digitalRead(pins[i]);
    states[i] = ps;
  }

  SerialBT.begin("MIDI-Controller");
  SerialBT.setPin("1");
  SerialBT.register_callback(BT_EventHandler);

  set_LED(LED::GREEN);
  leds.begin();
}

void loop()
{

  u_int8_t pedal_nr;

  for (u_int8_t pin_nr : pins)
  {
    pedal_nr = pin_routings[pin_nr];

    bool state = (bool)digitalRead(pin_nr);

    if (check_signal(pedal_nr, state))
    {
      set_LED(LED::GREEN);

      sendOutput(&routings[pedal_nr], state);
    }
  }

  cycle_LED();
  if (cfg_updated)
  {
    load_config();
    cfg_updated = false;
  }
}