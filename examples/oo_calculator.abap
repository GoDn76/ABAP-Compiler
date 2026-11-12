REPORT ztest_20.

CLASS lcl_calculator DEFINITION.
  PUBLIC SECTION.
    METHODS: add_numbers IMPORTING iv_a          TYPE i
                                   iv_b          TYPE i
                         RETURNING VALUE(rv_res) TYPE i.
ENDCLASS.

CLASS lcl_calculator IMPLEMENTATION.
  METHOD add_numbers.
    rv_res = iv_a + iv_b.
  ENDMETHOD.
ENDCLASS.

START-OF-SELECTION.
  DATA(lo_calc) = NEW lcl_calculator( ).
  DATA(lv_sum) = lo_calc->add_numbers( iv_a = 25 iv_b = 75 ).
  WRITE: / 'Class Method Output:', lv_sum.
