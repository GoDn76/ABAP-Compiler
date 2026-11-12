REPORT z_string_processing.

TYPES:
  BEGIN OF ty_text,
    raw TYPE string,
    first_part TYPE string,
    rest TYPE string,
    normalized TYPE string,
  END OF ty_text.

DATA ls_text TYPE ty_text.
DATA lv_length TYPE i.
DATA lv_lower TYPE string.
DATA lt_words TYPE TABLE OF string.

ls_text-raw = '  Abap,Cloud,Runtime  '.
TRANSLATE ls_text-raw TO UPPER CASE.
SPLIT ls_text-raw AT ',' INTO ls_text-first_part ls_text-rest.
CONCATENATE ls_text-first_part ls_text-rest INTO ls_text-normalized SEPARATED BY SPACE RESPECTING BLANKS.
CONDENSE ls_text-normalized.

lv_lower = to_lower( val = ls_text-normalized ).
lv_length = strlen( ls_text-normalized ).

WRITE ls_text-first_part.
WRITE ls_text-rest.
WRITE ls_text-normalized.
WRITE lv_lower.
WRITE lv_length.

TRANSLATE ls_text-normalized TO LOWER CASE.
WRITE ls_text-normalized.
