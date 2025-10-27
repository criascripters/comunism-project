"use client";

import type React from "react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import Link from "next/link";
import { useState } from "react";

export default function AuthPage() {
  const [isLogin, setIsLogin] = useState(true);

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = new FormData(e.currentTarget as HTMLFormElement);
    const email = String(form.get("email") || "").trim();
    const password = String(form.get("password") || "");

    if (isLogin) {
      // chamada simples à API /api/login
      fetch("/api/login", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email, password }),
      })
        .then(async (res) => {
          const json = await res.json();
          if (res.ok && json.ok) {
            // sucesso — comportamento mínimo: armazenar no localStorage e redirecionar para / (ou mostrar mensagem)
            try {
              localStorage.setItem("user", JSON.stringify(json.user));
            } catch {}
            window.location.href = "/";
          } else {
            const msg = json?.error || "Erro ao autenticar";
            alert("Falha no login: " + msg);
          }
        })
        .catch((err) => {
          console.error(err);
          alert("Erro de rede ao tentar logar");
        });
    } else {
      // signup flow mínimo (não cria usuário no servidor aqui)
      alert("Fluxo de criação de conta não implementado — use test@example.com / secret para login");
    }
  };

  return (
    <div className="min-h-screen bg-background flex flex-col">
      {/* Header */}
      <header className="border-b border-border bg-card">
        <div className="container mx-auto px-4 py-4">
          <Link href="/" className="flex items-center gap-2">
            <div className="h-10 w-10 rounded-lg bg-primary flex items-center justify-center">
              {/* <Zap className="h-6 w-6 text-primary-foreground" /> */}
              <img src="/mamaco.png" alt="Logo" className="h-6 w-6 text-primary-foreground" />
            </div>
            <span className="text-2xl font-bold tracking-tight">MONKEY BET</span>
          </Link>
        </div>
      </header>

      {/* Auth Form */}
      <div className="flex-1 flex items-center justify-center p-4">
        <Card className="w-full max-w-md">
          <CardHeader className="space-y-1">
            <CardTitle className="text-2xl font-bold text-center">{isLogin ? "Entrar" : "Criar Conta"}</CardTitle>
            <CardDescription className="text-center">
              {isLogin ? "Entre com suas credenciais para acessar sua conta" : "Crie sua conta para começar a apostar"}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              {!isLogin && (
                <div className="space-y-2">
                  <Label htmlFor="name">Nome Completo</Label>
                  <Input id="name" name="name" type="text" placeholder="Seu nome" required />
                </div>
              )}

              <div className="space-y-2">
                <Label htmlFor="email">Email</Label>
                <Input id="email" name="email" type="email" placeholder="seu@email.com" required />
              </div>

              <div className="space-y-2">
                <Label htmlFor="password">Senha</Label>
                <Input id="password" name="password" type="password" placeholder="••••••••" required />
              </div>

              {!isLogin && (
                <div className="space-y-2">
                  <Label htmlFor="confirmPassword">Confirmar Senha</Label>
                  <Input id="confirmPassword" name="confirmPassword" type="password" placeholder="••••••••" required />
                </div>
              )}

              {isLogin && (
                <div className="flex items-center justify-end">
                  <Link href="#" className="text-sm text-primary hover:underline">
                    Esqueceu a senha?
                  </Link>
                </div>
              )}

              <Button type="submit" className="w-full bg-primary hover:bg-primary/90 h-11">
                {isLogin ? "Entrar" : "Criar Conta"}
              </Button>
            </form>

            <div className="mt-6 text-center text-sm">
              <span className="text-muted-foreground">{isLogin ? "Não tem uma conta?" : "Já tem uma conta?"} </span>
              <button
                type="button"
                onClick={() => setIsLogin(!isLogin)}
                className="text-primary hover:underline font-medium"
              >
                {isLogin ? "Criar conta" : "Entrar"}
              </button>
            </div>

            {!isLogin && (
              <p className="mt-4 text-xs text-center text-muted-foreground leading-relaxed">
                Ao criar uma conta, você concorda com nossos Termos de Serviço e Política de Privacidade. Jogue com
                responsabilidade. +18
              </p>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
