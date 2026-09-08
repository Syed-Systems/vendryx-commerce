# Frontend Mahal Slot Ledger

**Frontend:** `apps/storefront/packages/frontend-mahal`  
**Contract:** 45/45 slots  
**Fork debt:** 0 files, 0 Medusa UI imports

`ported` means the supplied storefront screenshots directly govern the component's visible DOM.
`derived` means the references did not contain that contract surface, so it was drawn from the same
square-edged, hairline-divided, image-led Vendryx system. `forked` is a failure state and is absent.

| Contract slot | Status | Evidence |
|---|---|---|
| AccountLayout | derived | Generated mobile account reference and catalog tab geometry |
| AddressBook | derived | Account row and square field language |
| BrandFeatures | derived | Omitted because no sourced merchant claims are provided |
| CartMismatchBanner | derived | Flat full-width transactional alert |
| CartTemplate | ported | Generated cart reference and supplied product/cart visual cadence |
| CartTotals | ported | Product screenshot totals/footer rhythm and generated cart reference |
| CategoryGrid | ported | Supplied fashion home shop-by-style grid |
| CategoryTemplate | ported | Supplied men's and alternate clothing listings |
| Checkbox | derived | Checkout square-control language |
| ChevronDown | ported | Header, option, and filter disclosure glyphs |
| CollectionTemplate | ported | Supplied catalog hierarchy, adapted to collection data |
| CountrySelect | derived | Generated checkout reference |
| DiscountCode | ported | Generated cart disclosure |
| Divider | ported | Hairlines throughout all supplied screens |
| ErrorMessage | derived | Generated checkout inline-error state |
| Footer | ported | Supplied black multi-column footer |
| FreeShippingPriceNudge | derived | Intentionally omitted without a sourced threshold |
| Hero | ported | Supplied fashion home campaign hero |
| Input | derived | Generated checkout and account references |
| InteractiveLink | ported | Underlined editorial actions in supplied home/product screens |
| ItemsPreviewTemplate | ported | Generated cart drawer/checkout item layout |
| LoginTemplate | derived | Generated account reference and square panel system |
| LookbookStory | ported | Supplied bridal edit campaign banner |
| Nav | ported | Supplied jewelry mega-menu header and sticky product header |
| NewsletterSection | derived | Intentionally omitted until a working subscription action is provided |
| OrderCompletedTemplate | derived | Generated confirmation/account visual system |
| OrderDetailsTemplate | derived | Generated account and cart line-item system |
| OrderOverview | derived | Generated mobile account order rows |
| Overview | derived | Generated mobile account details |
| ProductTemplate | ported | Supplied Black Wallflower product page |
| ProfileBillingAddress | derived | Generated account and checkout field system |
| ProfileEmail | derived | Generated account detail rows |
| ProfileName | derived | Generated account detail rows |
| ProfilePassword | derived | Generated account detail rows |
| ProfilePhone | derived | Generated account detail rows |
| RadioIndicator | derived | Generated checkout selection controls |
| SkeletonCardDetails | derived | Neutral product media wells from supplied listings |
| SkeletonCartPage | derived | Cart grid and neutral media-well system |
| SkeletonOrderConfirmed | derived | Order confirmation layout system |
| Spinner | derived | Minimal monochrome transactional feedback |
| StoreTemplate | ported | Supplied men's and alternate clothing listings |
| SubmitButton | ported | Square black purchase buttons in supplied product page |
| TransferActions | derived | Generated account/order control language |
| TransferImage | derived | Vendryx identity and square media language |
| TransferRequestForm | derived | Generated account/order form language |

Additional loader-facing components—product actions, related products, product rail, paginated
products, sort controls, cart dropdown, and payment marks—use the same original DOM system. Product,
order, customer, money, region, and collection facts render only from core props. Static fixture facts
remain confined to the Vite harness.
