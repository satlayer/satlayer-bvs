import { NextResponse } from "next/server";

export async function GET() {
  const envKeys = Object.keys(process.env).sort();
  return NextResponse.json({
    version: "1.0.0",
    build: process.env.VERCEL_GIT_COMMIT_SHA?.substring(0, 8),
    envKeyCount: envKeys.length,
    envKeys: envKeys,
  });
}
