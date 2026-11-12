@EndUserText.label: 'Carrier projection for ABAPC local practice'
define view entity ZI_CARRIER_PRACTICE
  as select from SCARR
{
  key CARRID    as CarrierId,
      CARRNAME  as CarrierName,
      CURRCODE  as CurrencyCode
}
