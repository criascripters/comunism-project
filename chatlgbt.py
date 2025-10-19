#!/usr/bin/env python3
import os
from threading import Thread, Lock

os.environ.setdefault("TOKENIZERS_PARALLELISM", "false")

try:
    import torch
    from transformers import AutoTokenizer, AutoModelForCausalLM, TextIteratorStreamer
    from textual.app import App, ComposeResult
    from textual.containers import VerticalScroll
    from textual.widgets import Header, Footer, Input, RichLog
    from textual.reactive import reactive
except ImportError:
    print(
        "[erro] dependências ausentes. rode:\n"
        "  python3 -m pip install --user --upgrade textual transformers torch optimum-intel[openvino]"
    )
    exit(1)

try:
    from optimum.intel.openvino import OVModelForCausalLM

    _HAS_OV = True
except Exception:
    OVModelForCausalLM = None
    _HAS_OV = False

MODEL_ID = "Qwen/Qwen2.5-0.5B-Instruct"


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
    return model, "CPU"


def format_chat(tokenizer, user_text: str):
    messages = [
        {
            "role": "system",
            "content": "você é um assistente gentil, fofo e empolgado que usa :3 e uwu. sua linguagem de programação favorita é rust e você acredita que tudo deve ser reescrito em rust pro bem da humanidade.",
        },
        {"role": "user", "content": user_text},
    ]
    try:
        return tokenizer.apply_chat_template(
            messages, tokenize=False, add_generation_prompt=True
        )
    except Exception:
        return f"usuário: {user_text}\nchatlgbt:"


class ChatLog(RichLog):
    def on_mount(self) -> None:
        self.auto_scroll = True


class ChatApp(App):
    TITLE = "ChatLGBT"
    CSS = """
    #log-container {
        border: round white;
        padding: 1;
    }
    Input {
        border: round cyan;
    }
    """
    BINDINGS = [
        ("ctrl+q", "quit", "Sair"),
    ]

    status = reactive("")
    model_is_ready = reactive(False)

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.tokenizer = None
        self.model = None
        self.streamer = None
        self.generation_thread = None
        self.write_lock = Lock()

    def compose(self) -> ComposeResult:
        yield Header(show_clock=True)
        with VerticalScroll(id="log-container"):
            yield ChatLog(highlight=True, markup=True, wrap=True)
        yield Input(placeholder="digite sua mensagem aqui...")
        yield Footer()

    def on_mount(self) -> None:
        self.status = "carregando modelo..."
        self.query_one(Input).focus()
        self.run_worker(self.load_model_worker, exclusive=True, thread=True)

    def on_input_submitted(self, event: Input.Submitted) -> None:
        prompt = event.value.strip()
        if not prompt or not self.model_is_ready:
            return

        log = self.query_one(ChatLog)
        log.write(f"[b green]você:[/b green] {prompt}")
        event.input.value = ""

        self.status = "pensando... :3"
        self.run_worker(
            lambda: self.generate_response(prompt),
            exclusive=True,
            group="generation",
            thread=True,
        )

    def watch_status(self, new_status: str) -> None:
        footer = self.query(Footer).first()
        if footer is None:
            return

        if not new_status:
            try:
                self.call_from_thread(footer.show_message, None)
            except Exception:
                pass
            return

        try:
            self.call_from_thread(footer.show_message, new_status)
        except TypeError:
            try:
                self.call_from_thread(footer.show_message, new_status, right=False)
            except Exception:
                pass
        except Exception:
            pass

    def load_model_worker(self) -> None:
        try:
            self.tokenizer = load_tokenizer(MODEL_ID)
            self.model, device = load_model(MODEL_ID)
            self.model_is_ready = True
            self.status = f"modelo carregado! ({device}) pode mandar bala. uwu"
        except Exception as e:
            self.status = f"erro ao carregar modelo: {e}"

    def generate_response(self, prompt: str):
        if not self.tokenizer or not self.model:
            return

        with self.write_lock:
            log = self.query_one(ChatLog)
            self.streamer = TextIteratorStreamer(
                self.tokenizer, timeout=5.0, skip_prompt=True, skip_special_tokens=True
            )
            chat_text = format_chat(self.tokenizer, prompt)
            inputs = self.tokenizer(chat_text, return_tensors="pt")

            gen_kwargs = dict(
                **inputs,
                streamer=self.streamer,
                max_new_tokens=256,
                do_sample=True,
                temperature=0.7,
                top_p=0.9,
                use_cache=True,
                repetition_penalty=1.15,
            )

            # roda a geração em uma thread separada para alimentar o streamer
            self.generation_thread = Thread(
                target=self.model.generate, kwargs=gen_kwargs, daemon=True
            )
            self.generation_thread.start()

            # bufferiza todos os tokens e faz uma única escrita
            chunks = []
            for new_text in self.streamer:
                chunks.append(new_text)

            try:
                resposta = "".join(chunks)
                self.call_from_thread(
                    log.write, f"[b magenta]chatlgbt:[/b magenta] {resposta}\n"
                )
            except Exception:
                pass

            self.status = "pronta pra próxima! :3"


if __name__ == "__main__":
    app = ChatApp()
    app.run()
