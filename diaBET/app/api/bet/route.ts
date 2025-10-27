import { NextResponse } from "next/server";

const EXTERNAL_HOST = "v3.football.api-sports.io";
const DEFAULT_API_KEY = process.env.FOOTBALL_API_KEY || "eba7e4cb9fa69d7bdc48da21ddca017c";

export async function GET(request: Request) {
  try {
    const url = new URL(request.url);
    const params = url.search;

    const externalUrl = `https://${EXTERNAL_HOST}/fixtures${params}`;

    const res = await fetch(externalUrl, {
      method: "GET",
      headers: {
        "User-Agent": "comunism-project/1.0",
        "x-rapidapi-host": EXTERNAL_HOST,
        "x-rapidapi-key": DEFAULT_API_KEY,
      },
    });

    const data = await res.json();

    return NextResponse.json({ ok: true, data });
  } catch (err) {
    return NextResponse.json({ ok: false, error: String(err) }, { status: 500 });
  }
}
