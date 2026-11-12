/* EFR32MG24 OLED demo: cycles the moq.dev mark, the rind wordmark, and one
 * benchmark screen on an SSD1306 (128x64, I2C 0x3C, I2C1, PC4 SDA, PC5 SCL).
 * Wire.begin() uses the core's default I2C pins; pass pins if they differ.
 * Frames come from tools/gen_frames.py (frames.h, not committed).
 * Serial at 115200 prints the probe result and each frame change. */
#include <Arduino.h>
#include <Wire.h>
#include "frames.h"

static uint8_t oled_addr = 0;
static const uint32_t FRAME_MS = 4000;

static bool ack(uint8_t a) {
  Wire.beginTransmission(a);
  return Wire.endTransmission() == 0;
}

static void cmd(uint8_t c) {
  Wire.beginTransmission(oled_addr);
  Wire.write(0x00);
  Wire.write(c);
  Wire.endTransmission();
}

static void oled_init(void) {
  static const uint8_t seq[] = {
    0xAE, 0xD5, 0x80, 0xA8, 0x3F, 0xD3, 0x00, 0x40, 0x8D, 0x14,
    0x20, 0x00, 0xA1, 0xC8, 0xDA, 0x12, 0x81, 0xCF, 0xD9, 0xF1,
    0xDB, 0x40, 0xA4, 0xA6, 0x2E, 0xAF };
  for (size_t i = 0; i < sizeof(seq); i++) cmd(seq[i]);
}

static void oled_blit(const uint8_t *buf) {
  cmd(0x21); cmd(0); cmd(127);
  cmd(0x22); cmd(0); cmd(7);
  for (int i = 0; i < 1024; i += 16) {
    Wire.beginTransmission(oled_addr);
    Wire.write(0x40);
    Wire.write(buf + i, 16);
    Wire.endTransmission();
  }
}

void setup() {
  Serial.begin(115200);
  pinMode(LED_BUILTIN, OUTPUT);
  Wire.begin();
  Wire.setClock(400000);
  delay(50);
  if (ack(0x3C)) oled_addr = 0x3C;
  else if (ack(0x3D)) oled_addr = 0x3D;
  if (oled_addr) {
    oled_init();
  }
}

void loop() {
  static uint8_t f = 0;
  static uint32_t last = 0;
  static bool first = true;
  if (!oled_addr) {
    /* No OLED: say so every second and blink. */
    Serial.println("{\"app\":\"oled-demo\",\"oled\":\"absent\",\"probed\":[\"0x3c\",\"0x3d\"]}");
    digitalWrite(LED_BUILTIN, !digitalRead(LED_BUILTIN));
    delay(1000);
    return;
  }
  if (first || millis() - last >= FRAME_MS) {
    first = false;
    last = millis();
    oled_blit(demo_frames[f]);
    Serial.print("{\"app\":\"oled-demo\",\"oled\":\"0x");
    Serial.print(oled_addr, HEX);
    Serial.print("\",\"frame\":");
    Serial.print(f);
    Serial.println("}");
    f = (f + 1) % DEMO_FRAME_COUNT;
  }
  delay(10);
}
