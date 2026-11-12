REPORT zoo_inheritance.

CLASS lcl_base DEFINITION.
  PUBLIC SECTION.
    DATA mv_value TYPE i.
    METHODS get_value RETURNING VALUE(rv_value) TYPE i.
ENDCLASS.

CLASS lcl_base IMPLEMENTATION.
  METHOD get_value.
    rv_value = mv_value.
  ENDMETHOD.
ENDCLASS.

CLASS lcl_child DEFINITION INHERITING FROM lcl_base.
  PUBLIC SECTION.
    METHODS get_value REDEFINITION.
ENDCLASS.

CLASS lcl_child IMPLEMENTATION.
  METHOD get_value.
    rv_value = mv_value + 10.
  ENDMETHOD.
ENDCLASS.

START-OF-SELECTION.
  DATA lo_base TYPE REF TO lcl_base.
  lo_base = NEW lcl_child( ).
  lo_base->mv_value = 32.
  WRITE: / 'Dynamic dispatch:', lo_base->get_value( ).
