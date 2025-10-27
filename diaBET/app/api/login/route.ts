import { NextResponse } from "next/server";

type User = {
  email: string;
  password: string; // sem criptografia, conforme solicitado
  name: string;
};

// lista simples em memória (perdida ao reiniciar) — propósito didático
const USERS: User[] = [{ email: "test@example.com", password: "secret", name: "Usuário Teste" }];

export async function POST(request: Request) {
  try {
    const body = await request.json();
    const { email, password } = body as { email?: string; password?: string };

    if (!email || !password) {
      return NextResponse.json({ ok: false, error: "missing_fields" }, { status: 400 });
    }

    const user = USERS.find((u) => u.email === email && u.password === password);
    if (!user) {
      return NextResponse.json({ ok: false, error: "invalid_credentials" }, { status: 401 });
    }

    // sem sessão real — retornamos apenas info básica
    return NextResponse.json({ ok: true, user: { email: user.email, name: user.name } });
  } catch (err) {
    return NextResponse.json({ ok: false, error: "invalid_json" }, { status: 400 });
  }
}
