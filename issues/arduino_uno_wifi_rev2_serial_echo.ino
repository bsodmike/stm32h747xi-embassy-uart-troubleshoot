#ifndef outputSerial
#define outputSerial Serial1
#endif

int incomingByte = 0;               // for incoming serial data

void setup() {
    outputSerial.begin(9600);             // opens serial port, sets data rate to 9600 bps

    pinMode(LED_BUILTIN, OUTPUT);   // Green

    // while (Serial.available() <= 0) {
    //   digitalWrite(LED_BUILTIN, !digitalRead(LED_BUILTIN));
    //   delay(100);
    // }

    // if (Serial.available() > 0) {
    //   Serial.println("Starting echo.");
    // }
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
  // outputSerial.write("ATB\r\n");

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
  
  delay(2000);
}
