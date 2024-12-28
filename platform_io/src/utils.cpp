#include "utils.hpp"

void first_config()
{
  cfg.begin("config", false);

  cfg.putBool("init", true);
  for (u_int8_t i = 1; i <= AMT_PEDALS; i++)
  {
    Command cmd = Command{0, 100 + i, 0xff, 0xff};

    cfg.putBytes(std::to_string(i).c_str(), &cmd, sizeof(Command));
  }
  cfg.end();
}

void load_config()
{
  cfg.begin("config", true);
  Command cmd;
  for (u_int8_t i = 1; i <= AMT_PEDALS; i++)
  {
    // u_int8_t target = cfg.getUChar(std::to_string(i).c_str(), 0);
    cfg.getBytes(std::to_string(i).c_str(), &cmd, sizeof(Command));
    routings[i] = cmd;
  }

  cfg.end();
}

void send_config()
{
  int index = 0;

  for (auto &it : routings)
  {
    bt_output_buffer[index] = it.first;
    u_int8_t *bytes = (u_int8_t *)(void *)&it.second;

    for (int i = 0; i < sizeof(Command); i++)
    {
      bt_output_buffer[index + 1 + i] = bytes[i];
    }

    index += sizeof(Command) + 1;
  }
  SerialBT.write(bt_output_buffer, AMT_PEDALS * sizeof(Command) + AMT_PEDALS);
}

void update_config()
{
  int index = 1;

  cfg.begin("config", false);

  u_int8_t pedal;
  Command *cmd;

  while (true)
  {
    if (bt_input_buffer[index] == 0x00)
    {
      break;
    }
    pedal = bt_input_buffer[index];
    cmd = (Command *)bt_input_buffer[index + 1];

    cfg.putBytes(std::to_string(pedal).c_str(), cmd, sizeof(Command));

    index += sizeof(Command) + 1;
  }
  cfg.end();

  cfg_updated = true;
}

void heartbeat_response()
{
  bt_output_buffer[0] = 2;
  SerialBT.write(bt_output_buffer, 1);
}

void process_input()
{
  u_int8_t first = bt_input_buffer[0];

  switch (first)
  {
  case 0:
    send_config();
    break;
  case 1:
    update_config();
    break;
  case 2:
    heartbeat_response();
    break;
  }
}

void BT_EventHandler(esp_spp_cb_event_t event, esp_spp_cb_param_t *param)
{
  if (event == ESP_SPP_START_EVT)
  {
  }
  else if (event == ESP_SPP_SRV_OPEN_EVT)
  {
    set_LED(LED::BLUE);
  }
  else if (event == ESP_SPP_CLOSE_EVT)
  {
    set_LED(LED::RED);
  }
  else if (event == ESP_SPP_DATA_IND_EVT)
  {
    int index = 0;
    while (SerialBT.available())
    {
      int incoming = SerialBT.read();
      bt_input_buffer[index] = incoming;
      index++;
    }
    bt_input_buffer[index] = 0x00;
    process_input();

    set_LED(LED::BLUE);
  }
}

bool check_signal(u_int8_t pedal_nr, bool input)
{
  pin_state *current = &states[pedal_nr - 1];

  if (input == current->state)
  {
    return false;
  }

  current->signal++;

  if (current->signal > TOLERANCE_CAP)
  {
    current->state = input;
    current->signal = 0;
    return true;
  }

  return false;
}

void set_LED(LED value)
{
  switch (value)
  {
  // R<->G
  case LED::RED:
    color = {0x00, 0xFF, 0x00};
    break;
  case LED::GREEN:
    color = {217, 41, 30};
    break;
  case LED::BLUE:
    color = {0, 31, 0xFF};
    break;
  }
}

void cycle_LED()
{
  if (!color.empty())
  {
    for (u_int16_t i = 0; i < LED_COUNT; i++)
    {
      leds.setPixelColor(i, brightness * color[0], brightness * color[1], brightness * color[2]);
    }

    leds.show();
    if (brightness >= 0.98)
    {
      LED_down = true;
    }
    else if (brightness <= 0.3)
    {
      LED_down = false;
    }

    if (LED_down)
    {
      brightness -= BRIGHTNESS_STEP;
    }
    else
    {
      brightness += BRIGHTNESS_STEP;
    }
  }
}