CLASS zcl_cloud_classrun DEFINITION
  PUBLIC
  FINAL
  CREATE PUBLIC.

  PUBLIC SECTION.
    INTERFACES if_oo_adt_classrun.
ENDCLASS.

CLASS zcl_cloud_classrun IMPLEMENTATION.

  METHOD if_oo_adt_classrun~main.
    DATA lv_total TYPE i.
    lv_total = 10 + 20.
    out->write( lv_total ).
  ENDMETHOD.

ENDCLASS.
