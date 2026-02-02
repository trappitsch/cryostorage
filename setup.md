# Setup for the project 

## ReTerminal / Host computer

- IP: 192.168.1.1

## Moxa 

The moxa we are using is a NPort 5650-8-DT. 
The following ports are configured:

IP setup (static):

- IP: 192.168.1.2 
- Subnet mask: 255.255.255.0

- Port 1: 
  - Agilent 4UHV Ion pump controller 
  - Serial setup: 38_400, 8, N, 1
  - Mode: TCP Server
  - IP Port: 4001

- Port 3:
  - Sunpower CryoTel GT Cryocooler
  - Serial setup: 4_800, 8, N, 1
  - Mode: TCP Server
  - IP Port: 4003

## Pfeiffer HiCube Pump stand

The HiCube is connected to the second LAN port of the Moxa.
The host computer is connected to the other LAN port of the Moxa.
With the static IP setup as we currently have it, 
the HiCube can be accessed from the host computer, i.e., 
the Moxa acts as a switch.

- IP: 192.168.1.100

