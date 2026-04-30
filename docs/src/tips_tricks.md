# Tips &amp; Tricks

Below are some tips and tricks that are not directly visible from the GUI.

## Sample management

- You can swap the position of two samples by simply dragging and dropping
  one sample onto another.
  This swaps their position. If the second sample position is empty,
  it still swaps the position and the first sample position is now empty.
- To delete a sample, simply click on the position, then click "Clear" and "Ok".
- If you add a new sample or change the position of a sample,
  the current sample positions are automatically saved in the configuration file.
  Furthermore: Everytime a sample is changed/added/moved, a timestamped backup
  of the previous configuration file is saved in the `archive` folder under
  `$HOME/.cryostorage`. This backup is meant as a last-resort in case samples
  get mixed up.

## Pressure, temperature history

While the plots in the GUI only show the last 24h, all recorded data are stored
in the `$HOME/.cryostorage` folder on disk as `csv` files.
Over time, pressure and temperature histories are rotated into the archive folder
and not kept forever.

## Logs

If communication with any of the instruments fails or returns bad data,
the incident is logged.
The last 100 log entries can be found in the settings tab.
Logs are also stored in the `$HOME/.cryostorage` folder on the disk.
They are rotated over time into the archive folder and not kept forever.
