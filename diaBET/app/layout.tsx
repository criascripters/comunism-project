import type React from "react"
import type { Metadata } from "next"
import { Ubuntu } from "next/font/google"
import { Analytics } from "@vercel/analytics/next"
import "./globals.css"

const ubuntu = Ubuntu({
  weight: ["300", "400", "500", "700"],
  subsets: ["latin"],
  variable: "--font-ubuntu",
})

export const metadata: Metadata = {
  title: "Monkey Bet - Live Sports Betting",
  description: "Experience the thrill of live in-play betting on your favorite sports",
  generator: "v0.app",
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en">
      <body className={`${ubuntu.variable} font-sans antialiased`}>
        {children}
        <Analytics />
      </body>
    </html>
  )
}
