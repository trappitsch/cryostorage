# **pcb_controller - Design Notes**

## **Zenerdiodes for voltage regulation:**

- The current in all of the 24V read lines is approx. 2mA
- We thus use the BZX84C3V6 3.6 V zenerdiode, as at ~2mA, this should give us a voltage of approx 3.2 V
- This was tested on 2026-02-12 with the old version of hardware by ripping out the voltage divider resistors and using the zenerdiode as r

---
*KiNotes - PCBtools.xyz*

