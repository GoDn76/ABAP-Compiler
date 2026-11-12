SELECT FROM SCARR
  FIELDS CARRID, CARRNAME
  WHERE CURRCODE = 'USD'
  INTO TABLE @DATA(lt_carriers).

WRITE 'USD carriers:', LINES( lt_carriers ).
LOOP AT lt_carriers INTO DATA(ls_carrier).
  WRITE ls_carrier-CARRID, ls_carrier-CARRNAME.
ENDLOOP.
