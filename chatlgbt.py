#!/usr/bin/env python3
import os
import sys
import subprocess
import argparse
from threading import Thread

os.environ.setdefault("TOKENIZERS_PARALLELISM", "false")

DEPS = [
    "transformers>=4.51.0",
    "optimum-intel[openvino]>=1.17.0",  # opcional, usado se disponível
    "torch",
]


def ensure_deps():
    """
    checa dependências e avisa se alguma estiver faltando.
    não tenta instalar automaticamente (evita pep 668 em sistemas como arch).
    """
    missing = []
    for spec in DEPS:
        pkg = spec.split(">", 1)[0].split("[", 1)[0]
        try:
            __import__(pkg.replace("-", "_"))
        except Exception:
            missing.append(spec)
    if missing:
        print("[aviso] dependências ausentes detectadas:")
        for m in missing:
            print(f"  - {m}")
        print(
            "\npara instalar manualmente, rode:\n"
            "  python3 -m pip install --user --upgrade "
            + " ".join(missing)
            + "\n\nou, se quiser arriscar no sistema:\n"
            "  sudo python3 -m pip install --break-system-packages " + " ".join(missing)
        )
        print("\nseguindo mesmo assim...\n")


ensure_deps()

import torch
from transformers import AutoTokenizer, AutoModelForCausalLM, TextIteratorStreamer

# tenta importar o modelo otimizado, senao usa AutoModelForCausalLM
try:
    from optimum.intel.openvino import OVModelForCausalLM  # type: ignore

    _HAS_OV = True
except Exception:
    try:
        from optimum.intel import OVModelForCausalLM  # type: ignore

        _HAS_OV = True
    except Exception:
        OVModelForCausalLM = None
        _HAS_OV = False

HEADER = r"""
    __  __ __   ____  ______  _       ____  ____   ______
   /  ]|  |  | /    ||      || |     /    ||    \ |      |
  /  / |  |  ||  o  ||      || |    |   __||  o  )|      |
 /  /  |  _  ||     ||_|  |_|| |___ |  |  ||     ||_|  |_|
/   \_ |  |  ||  _  |  |  |  |     ||  |_ ||  O  |  |  |
\     ||  |  ||  |  |  |  |  |     ||     ||     |  |  |
 \____||__|__||__|__|  |__|  |_____||___,_||_____|  |__|

"""


def build_argparser():
    p = argparse.ArgumentParser(description="chat minimalista com streaming")
    p.add_argument("prompt", nargs="*", help="prompt one-shot (opcional)")
    p.add_argument(
        "--model",
        default="Qwen/Qwen2.5-0.5B-Instruct",
        help="modelo hf (default: Qwen/Qwen2.5-0.5B-Instruct)",
    )
    p.add_argument(
        "--device",
        default="GPU",
        choices=["GPU", "CPU"],
        help="dispositivo preferido (GPU/CPU). o loader faz fallback se necessário.",
    )
    p.add_argument("--max-new-tokens", type=int, default=128)
    p.add_argument("--temperature", type=float, default=0.7)
    p.add_argument("--top-p", type=float, default=0.9)
    p.add_argument("--top-k", type=int, default=50)
    p.add_argument("--no-stream", action="store_true", help="desliga streaming")
    return p


def load_tokenizer(model_id: str):
    tok = AutoTokenizer.from_pretrained(model_id, trust_remote_code=True)
    if tok.pad_token is None:
        if getattr(tok, "eos_token", None) is not None:
            tok.pad_token = tok.eos_token
        else:
            tok.add_special_tokens({"pad_token": "<|pad|>"})
    return tok


def load_model(model_id: str, device: str = "GPU"):
    if _HAS_OV and OVModelForCausalLM is not None:
        try:
            model = OVModelForCausalLM.from_pretrained(
                model_id, export=True, device=device, trust_remote_code=True
            )
            return model, device
        except Exception:
            try:
                model = OVModelForCausalLM.from_pretrained(
                    model_id, export=True, device="CPU", trust_remote_code=True
                )
                return model, "CPU"
            except Exception:
                pass
    model = AutoModelForCausalLM.from_pretrained(model_id, trust_remote_code=True)
    model.eval()
    try:
        if getattr(model, "generation_config", None) is not None:
            if model.generation_config.eos_token_id is None:
                model.generation_config.eos_token_id = None
    except Exception:
        pass
    return model, "CPU"


def format_chat(tokenizer, user_text: str, think: bool = False):
    messages = [
        {
            "role": "system",
            "content": "você é um assistente gentil, fofo e empolgado que usa :3 e uwu. sua linguagem de programação favorita é rust e você acredita que tudo deve ser reescrito em rust pro bem da humanidade.",
        },
        {"role": "user", "content": user_text},
    ]
    try:
        text = tokenizer.apply_chat_template(
            messages, tokenize=False, add_generation_prompt=True, enable_thinking=think
        )
    except TypeError:
        try:
            text = tokenizer.apply_chat_template(
                messages, tokenize=False, add_generation_prompt=True
            )
        except Exception:
            text = f"usuário: {user_text}\nchatlgbt:"
    except Exception:
        text = f"usuário: {user_text}\nchatlgbt:"
    return text


def generate_stream(
    model,
    tokenizer,
    prompt_text: str,
    max_new_tokens: int,
    temperature: float,
    top_p: float,
    top_k: int,
    stream: bool,
):
    inputs = tokenizer(prompt_text, return_tensors="pt")
    eos_id = getattr(tokenizer, "eos_token_id", None)
    if eos_id is None:
        try:
            eos_id = (
                tokenizer.convert_tokens_to_ids(tokenizer.eos_token)
                if getattr(tokenizer, "eos_token", None)
                else None
            )
        except Exception:
            eos_id = None

    gen_kwargs = dict(
        **inputs,
        max_new_tokens=max_new_tokens,
        do_sample=True,
        temperature=temperature,
        top_p=top_p,
        top_k=top_k,
        pad_token_id=getattr(tokenizer, "pad_token_id", None),
        eos_token_id=eos_id,
        use_cache=True,
        repetition_penalty=1.15,
    )

    if not stream:
        out_ids = model.generate(**gen_kwargs)
        decoded = tokenizer.decode(out_ids[0], skip_special_tokens=True)
        if decoded.lstrip().lower().startswith("chatlgbt:"):
            decoded = decoded.lstrip()[len("chatlgbt:") :].lstrip()
        print(decoded)
        return

    streamer = TextIteratorStreamer(
        tokenizer, timeout=5.0, skip_prompt=True, skip_special_tokens=True
    )
    gen_kwargs["streamer"] = streamer

    t = Thread(target=model.generate, kwargs=gen_kwargs, daemon=True)
    t.start()

    first_piece = True
    try:
        for piece in streamer:
            if first_piece:
                pl = piece.lstrip()
                if pl.lower().startswith("chatlgbt:"):
                    piece = pl[len("chatlgbt:") :].lstrip()
                print(piece, end="", flush=True)
                first_piece = False
            else:
                print(piece, end="", flush=True)
    except KeyboardInterrupt:
        print("\n[interrompido pelo usuário]\n")
    finally:
        t.join(timeout=10.0)
        if t.is_alive():
            print("\n[aviso] geração ainda ativa — encerrando.\n")
        print()


def main():
    args = build_argparser().parse_args()
    print(HEADER)

    model_id = args.model
    tokenizer = load_tokenizer(model_id)
    model, used_device = load_model(model_id, args.device)

    try:
        if (
            getattr(model, "generation_config", None) is not None
            and getattr(tokenizer, "eos_token_id", None) is not None
        ):
            model.generation_config.eos_token_id = tokenizer.eos_token_id
    except Exception:
        pass

    if args.prompt:
        user = " ".join(args.prompt).strip()
        chat_txt = format_chat(tokenizer, user)
        print("chatlgbt: ", end="", flush=True)
        generate_stream(
            model,
            tokenizer,
            chat_txt,
            max_new_tokens=args.max_new_tokens,
            temperature=args.temperature,
            top_p=args.top_p,
            top_k=args.top_k,
            stream=not args.no_stream,
        )
        return

    while True:
        try:
            user = input("você: ").strip()
        except (EOFError, KeyboardInterrupt):
            print("\nflw.")
            break
        if not user:
            continue
        if user.lower() in {"sair", "exit", "quit", "q"}:
            print("flw.")
            break

        chat_txt = format_chat(tokenizer, user)
        print("chatlgbt: ", end="", flush=True)
        try:
            generate_stream(
                model,
                tokenizer,
                chat_txt,
                max_new_tokens=args.max_new_tokens,
                temperature=args.temperature,
                top_p=args.top_p,
                top_k=args.top_k,
                stream=not args.no_stream,
            )
        except Exception as e:
            print(f"\n[erro] geração falhou: {e}\n")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"\n[erro fatal] {e}\n")
        raise
