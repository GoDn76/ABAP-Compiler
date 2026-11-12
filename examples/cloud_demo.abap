DATA lv_total TYPE i.
CONSTANTS c_limit TYPE i VALUE 10.

lv_total = 2 + 3 * 4.

WHILE lv_total < c_limit.
  lv_total = lv_total + 1.
ENDWHILE.

WRITE lv_total.
