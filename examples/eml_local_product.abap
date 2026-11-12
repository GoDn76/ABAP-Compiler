REPORT zeml_local_product.

" ABAPC local RAP/EML training subset.
" This is not connected to an SAP RAP business object.

MODIFY ENTITIES OF ZR_PRODUCT ENTITY Product
  CREATE FIELDS ( PRODUCT_ID PRODUCT_NAME CATEGORY PRICE CURRENCY )
  WITH VALUE #( ( PRODUCT_ID = 'TRAIN-400' PRODUCT_NAME = 'USB Hub' CATEGORY = 'ACCESSORY' PRICE = 2499 CURRENCY = 'INR' ) )
  MAPPED DATA(mapped)
  FAILED DATA(failed)
  REPORTED DATA(reported).

IF LINES( failed ) = 0.
  COMMIT ENTITIES.
  WRITE: / 'Product created:', 'TRAIN-400'.
ELSE.
  ROLLBACK ENTITIES.
  WRITE: / 'Product was not created'.
ENDIF.

READ ENTITIES OF ZI_PRODUCT ENTITY Product ALL FIELDS
  FROM VALUE #( ( PRODUCT_ID = 'TRAIN-400' ) )
  RESULT DATA(products)
  FAILED DATA(read_failed).

LOOP AT products INTO DATA(product).
  WRITE: / product-PRODUCT_NAME, product-PRICE.
ENDLOOP.
