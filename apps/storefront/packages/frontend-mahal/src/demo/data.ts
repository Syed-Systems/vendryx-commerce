export type FixtureProduct = {
  id: string
  brand: string
  title: string
  price: string
  image: string
  alternate?: string
  badge?: string
}

export const products: FixtureProduct[] = [
  {
    id: "black-wallflower",
    brand: "MASABA",
    title: "Black Wallflower Kurta",
    price: "BDT 23,800.00",
    image: "/media/kurta-black.jpg",
  },
  {
    id: "bottle-green",
    brand: "MASABA",
    title: "Bottle Green Buttercup Men's Kurta",
    price: "BDT 31,800.00",
    image: "/media/kurta-green.jpg",
  },
  {
    id: "aabro",
    brand: "PHULOKAY",
    title: "Aabro",
    price: "BDT 19,800.00",
    image: "/media/kurta-green.jpg",
  },
  {
    id: "printed-silk",
    brand: "ELEVEN BROTHERS",
    title: "Black Night Printed Silk Kurta",
    price: "BDT 11,900.00",
    badge: "SALE",
    image: "/media/kurta-black.jpg",
  },
  {
    id: "maroon",
    brand: "MASABA",
    title: "Maroon Chikankari Kurta Set",
    price: "BDT 28,500.00",
    image: "/media/lehenga-pink.jpg",
  },
  {
    id: "mustard",
    brand: "PHULOKAY",
    title: "Mustard Festive Kurta",
    price: "BDT 16,900.00",
    image: "/media/kurta-green.jpg",
  },
  {
    id: "blue",
    brand: "PHULOKAY",
    title: "Firozy Kurta Set",
    price: "BDT 15,800.00",
    image: "/media/kurta-green.jpg",
  },
  {
    id: "ivory",
    brand: "PHULOKAY",
    title: "Ivory Embroidered Kurta",
    price: "BDT 15,000.00",
    image: "/media/lehenga-pink.jpg",
  },
]

export const styles = [
  ["LEHENGAS", "/media/lehenga-pink.jpg"],
  ["SHERWANIS", "/media/kurta-black.jpg"],
  ["SAREES", "/media/campaign-hero.jpg"],
  ["KURTA SETS", "/media/kurta-green.jpg"],
] as const
