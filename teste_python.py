import random
import time

cores = ["\033[31m", "\033[32m", "\033[33m", "\033[34m", "\033[35m"]
mensagens = ["dev python recebe bem man confia", "seloco fizeram lib do pornhub... os cara tão na proatividade", "fazer implementaçao do zero? ta maluco num compensa, importa lib ai", "ssssssssssssssssssssssssssssss"]

while True:
    cor = random.choice(cores)
    msg = random.choice(mensagens)
    print(f"{cor}{msg}\033[0m")
    time.sleep(0.7)
