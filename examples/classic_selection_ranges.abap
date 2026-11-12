REPORT z_selection_ranges.

PARAMETERS p_upper AS CHECKBOX DEFAULT 'X'.
SELECT-OPTIONS s_carrid FOR scarr-carrid DEFAULT 'LH'.

AT SELECTION-SCREEN ON VALUE-REQUEST FOR s_carrid-low.
  WRITE 'LH'.
  WRITE 'SQ'.

START-OF-SELECTION.
  DATA lt_carriers TYPE TABLE OF scarr.

  SELECT FROM scarr
    FIELDS carrid, carrname, currcode
    WHERE carrid IN s_carrid
    INTO TABLE @lt_carriers.

  LOOP AT lt_carriers INTO DATA(ls_carrier).
    IF p_upper = 'X'.
      TRANSLATE ls_carrier-carrname TO UPPER CASE.
    ENDIF.
    WRITE ls_carrier-carrid, ls_carrier-carrname.
  ENDLOOP.
