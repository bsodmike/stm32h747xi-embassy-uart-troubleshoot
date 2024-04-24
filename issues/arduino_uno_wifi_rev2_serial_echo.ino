#ifndef outputSerial
#define outputSerial Serial1
#endif

int incomingByte = 0;               // For incoming serial data

void setup() {
    outputSerial.begin(115200);     // Opens serial port, sets data rate to 115200 bps

    pinMode(LED_BUILTIN, OUTPUT);   // Green
}

void toggle_pin(pin_size_t *pin) {
  digitalWrite(*pin, LOW);
  delay(100);
  digitalWrite(*pin, HIGH);
  delay(100);
}

void toggle_led_builtin() {
  digitalWrite(LED_BUILTIN, LOW);
  delay(100);
  digitalWrite(LED_BUILTIN, HIGH);
  delay(100);
}

void loop() {
  // send data only when you receive data:
  if (outputSerial.available() > 0) {
    toggle_led_builtin();

    // read the incoming byte:
    incomingByte = outputSerial.read();
  
    outputSerial.print("Got: ");
    // say what you got:
    outputSerial.print((char)incomingByte);
    outputSerial.println("");
  }
  else {
    toggle_led_builtin();
  }
  
  delay(100);
}
