REPORT zhello.

DATA lv_name TYPE string.
DATA lv_n TYPE i.

lv_name = 'ABAPC'.
lv_n = 3.

IF lv_n > 2.
  WRITE lv_name.
ELSE.
  WRITE 'small'.
ENDIF.
