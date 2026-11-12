DATA lv_i TYPE i.
DATA lv_sum TYPE i.

DO 5 TIMES.
  lv_i = lv_i + 1.
  CHECK lv_i < 4.
  lv_sum = lv_sum + lv_i.
ENDDO.

WRITE 'sum=', lv_sum.
