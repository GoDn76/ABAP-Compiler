REPORT ztest_17.

DATA: lv_status TYPE string VALUE 'E'.

DATA(lv_message) = SWITCH string( lv_status
  WHEN 'S' THEN 'Success'
  WHEN 'W' THEN 'Warning'
  WHEN 'E' THEN 'Error'
  ELSE 'Unknown'
).

WRITE: / 'Status Message:', lv_message.
