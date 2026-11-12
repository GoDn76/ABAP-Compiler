CLASS zcl_cloud_test3 DEFINITION
  PUBLIC
  FINAL
  CREATE PUBLIC.

  PUBLIC SECTION.

    TYPES:
      BEGIN OF ty_product,
        id    TYPE i,
        name  TYPE string,
        price TYPE i,
      END OF ty_product.

    INTERFACES if_oo_adt_classrun.

ENDCLASS.

CLASS zcl_cloud_test3 IMPLEMENTATION.

  METHOD if_oo_adt_classrun~main.

    DATA lt_products TYPE TABLE OF ty_product.
    DATA ls_product TYPE ty_product.
    DATA lv_total TYPE i.

    ls_product-id = 1.
    ls_product-name = 'Keyboard'.
    ls_product-price = 1200.
    APPEND ls_product TO lt_products.

    ls_product-id = 2.
    ls_product-name = 'Mouse'.
    ls_product-price = 700.
    APPEND ls_product TO lt_products.

    ls_product-id = 3.
    ls_product-name = 'Monitor'.
    ls_product-price = 15000.
    APPEND ls_product TO lt_products.

    lv_total = 0.

    LOOP AT lt_products INTO ls_product.
      lv_total = lv_total + ls_product-price.
    ENDLOOP.

    out->write( 'Total:' ).
    out->write( lv_total ).

  ENDMETHOD.

ENDCLASS.
