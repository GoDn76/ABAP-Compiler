CLASS zcl_cloud_table DEFINITION
  PUBLIC
  FINAL
  CREATE PUBLIC.

  PUBLIC SECTION.
    INTERFACES if_oo_adt_classrun.
ENDCLASS.

CLASS zcl_cloud_table IMPLEMENTATION.

  METHOD if_oo_adt_classrun~main.
    DATA lt_numbers TYPE TABLE OF i WITH EMPTY KEY.
    DATA lv_total TYPE i.

    APPEND 10 TO lt_numbers.
    APPEND 20 TO lt_numbers.
    APPEND 30 TO lt_numbers.

    LOOP AT lt_numbers INTO DATA(lv_number).
      lv_total = lv_total + lv_number.
    ENDLOOP.

    out->write( data = lv_total ).
  ENDMETHOD.

ENDCLASS.
