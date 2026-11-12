TYPES: BEGIN OF ty_person,
  id TYPE i,
  name TYPE string,
END OF ty_person.

DATA lt_people TYPE TABLE OF ty_person WITH EMPTY KEY.
APPEND VALUE #( id = 3 name = 'C' ) TO lt_people.
APPEND VALUE #( id = 1 name = 'A' ) TO lt_people.
APPEND VALUE #( id = 2 name = 'B' ) TO lt_people.

SORT lt_people BY id ASCENDING.
LOOP AT lt_people INTO DATA(ls_person).
  WRITE ls_person-id, ls_person-name.
ENDLOOP.
