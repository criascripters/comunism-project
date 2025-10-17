unit uBBPixCobRequest;

interface

uses
   SysUtils, DBXJSON, Generics.Collections, Classes;

type
   TCalendario = class
   private
      FExpiracao: Integer;
   public
      property Expiracao: Integer read FExpiracao write FExpiracao;
      function ToJSON: TJSONObject;
      procedure FromJSON(AJSON: TJSONObject);
      function HasData: Boolean;
   end;

   TDevedor = class
   private
      FCpf: string;
      FCnpj: string;
      FNome: string;
   public
      property Cpf: string read FCpf write FCpf;
      property Cnpj: string read FCnpj write FCnpj;
      property Nome: string read FNome write FNome;
      function ToJSON: TJSONObject;
      procedure FromJSON(AJSON: TJSONObject);
      function HasData: Boolean;
   end;

   TLoc = class
   private
      FId: Int64;
   public
      property Id: Int64 read FId write FId;
      function ToJSON: TJSONObject;
      procedure FromJSON(AJSON: TJSONObject);
      function HasData: Boolean;
   end;

   TRetiradaItem = class
   private
      FValor: string;
      FModalidadeAlteracao: Integer;
      FModalidadeAgente: string;
      FPrestadorDoServicoDeSaque: string;
   public
      property Valor: string read FValor write FValor;
      property ModalidadeAlteracao: Integer read FModalidadeAlteracao write FModalidadeAlteracao;
      property ModalidadeAgente: string read FModalidadeAgente write FModalidadeAgente;
      property PrestadorDoServicoDeSaque: string read FPrestadorDoServicoDeSaque write FPrestadorDoServicoDeSaque;
      function ToJSON: TJSONObject;
      procedure FromJSON(AJSON: TJSONObject);
      function HasData: Boolean;
   end;

   TRetirada = class
   private
      FSaque: TRetiradaItem;
      FTroco: TRetiradaItem;
   public
      constructor Create;
      destructor Destroy; override;
      property Saque: TRetiradaItem read FSaque write FSaque;
      property Troco: TRetiradaItem read FTroco write FTroco;
      function ToJSON: TJSONObject;
      procedure FromJSON(AJSON: TJSONObject);
      function HasData: Boolean;
   end;

   TValor = class
   private
      FOriginal: string;
      FModalidadeAlteracao: Integer;
      FRetirada: TRetirada;
   public
      constructor Create;
      destructor Destroy; override;
      property Original: string read FOriginal write FOriginal;
      property ModalidadeAlteracao: Integer read FModalidadeAlteracao write FModalidadeAlteracao;
      property Retirada: TRetirada read FRetirada write FRetirada;
      function ToJSON: TJSONObject;
      procedure FromJSON(AJSON: TJSONObject);
      function HasData: Boolean;
   end;

   TInfoAdicional = class
   private
      FNome: string;
      FValor: string;
   public
      property Nome: string read FNome write FNome;
      property Valor: string read FValor write FValor;
      function ToJSON: TJSONObject;
      procedure FromJSON(AJSON: TJSONObject);
      function HasData: Boolean;
   end;

   TBBPixCobRequest = class
   private
      FCalendario: TCalendario;
      FDevedor: TDevedor;
      FLoc: TLoc;
      FValor: TValor;
      FChave: string;
      FSolicitacaoPagador: string;
      FInfoAdicionais: TObjectList<TInfoAdicional>;
   public
      constructor Create;
      destructor Destroy; override;
      property Calendario: TCalendario read FCalendario write FCalendario;
      property Devedor: TDevedor read FDevedor write FDevedor;
      property Loc: TLoc read FLoc write FLoc;
      property Valor: TValor read FValor write FValor;
      property Chave: string read FChave write FChave;
      property SolicitacaoPagador: string read FSolicitacaoPagador write FSolicitacaoPagador;
      property InfoAdicionais: TObjectList<TInfoAdicional> read FInfoAdicionais write FInfoAdicionais;
      function ToJSON: TJSONObject;
      procedure FromJSON(AJSON: TJSONObject);
   end;

implementation

{ TCalendario }

function TCalendario.HasData: Boolean;
begin
   Result := FExpiracao > 0;
end;

function TCalendario.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FExpiracao > 0 then
      Result.AddPair('expiracao', TJSONNumber.Create(FExpiracao));
end;

procedure TCalendario.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('expiracao')) then
      FExpiracao := StrToInt(AJSON.Get('expiracao').JsonValue.Value);
end;

{ TDevedor }

function TDevedor.HasData: Boolean;
begin
   Result := (FCpf <> '') or (FCnpj <> '') or (FNome <> '');
end;

function TDevedor.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FCpf <> '' then
      Result.AddPair('cpf', TJSONString.Create(FCpf));
   if FCnpj <> '' then
      Result.AddPair('cnpj', TJSONString.Create(FCnpj));
   if FNome <> '' then
      Result.AddPair('nome', TJSONString.Create(FNome));
end;

procedure TDevedor.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('cpf')) then
      FCpf := AJSON.Get('cpf').JsonValue.Value;
   if Assigned(AJSON.Get('cnpj')) then
      FCnpj := AJSON.Get('cnpj').JsonValue.Value;
   if Assigned(AJSON.Get('nome')) then
      FNome := AJSON.Get('nome').JsonValue.Value;
end;

{ TLoc }

function TLoc.HasData: Boolean;
begin
   Result := FId > 0;
end;

function TLoc.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FId > 0 then
      Result.AddPair('id', TJSONNumber.Create(FId));
end;

procedure TLoc.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('id')) then
      FId := StrToInt64(AJSON.Get('id').JsonValue.Value);
end;

{ TRetiradaItem }

function TRetiradaItem.HasData: Boolean;
begin
   Result := (FValor <> '') or (FModalidadeAgente <> '') or (FPrestadorDoServicoDeSaque <> '');
end;

function TRetiradaItem.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FValor <> '' then
      Result.AddPair('valor', TJSONString.Create(FValor));
   if (FModalidadeAlteracao = 0) or (FModalidadeAlteracao = 1) then
      Result.AddPair('modalidadeAlteracao', TJSONNumber.Create(FModalidadeAlteracao));
   if FModalidadeAgente <> '' then
      Result.AddPair('modalidadeAgente', TJSONString.Create(FModalidadeAgente));
   if FPrestadorDoServicoDeSaque <> '' then
      Result.AddPair('prestadorDoServicoDeSaque', TJSONString.Create(FPrestadorDoServicoDeSaque));
end;

procedure TRetiradaItem.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('valor')) then
      FValor := AJSON.Get('valor').JsonValue.Value;
   if Assigned(AJSON.Get('modalidadeAlteracao')) then
      FModalidadeAlteracao := StrToInt(AJSON.Get('modalidadeAlteracao').JsonValue.Value);
   if Assigned(AJSON.Get('modalidadeAgente')) then
      FModalidadeAgente := AJSON.Get('modalidadeAgente').JsonValue.Value;
   if Assigned(AJSON.Get('prestadorDoServicoDeSaque')) then
      FPrestadorDoServicoDeSaque := AJSON.Get('prestadorDoServicoDeSaque').JsonValue.Value;
end;

{ TRetirada }

constructor TRetirada.Create;
begin
   FSaque := TRetiradaItem.Create;
   FTroco := TRetiradaItem.Create;
end;

destructor TRetirada.Destroy;
begin
   FSaque.Free;
   FTroco.Free;
   inherited;
end;

function TRetirada.HasData;
begin
   Result := FSaque.HasData or FTroco.HasData;
end;

function TRetirada.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FSaque.HasData then
      Result.AddPair('saque', FSaque.ToJSON);
   if FTroco.HasData then
      Result.AddPair('troco', FTroco.ToJSON);
end;

procedure TRetirada.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('saque')) then
      FSaque.FromJSON(TJSONObject(AJSON.Get('saque').JsonValue));
   if Assigned(AJSON.Get('troco')) then
      FTroco.FromJSON(TJSONObject(AJSON.Get('troco').JsonValue));
end;

{ TValor }

constructor TValor.Create;
begin
   FRetirada := TRetirada.Create;
end;

destructor TValor.Destroy;
begin
   FRetirada.Free;
   inherited;
end;

function TValor.HasData;
begin
   Result := (FOriginal <> '') or (FModalidadeAlteracao > -1) or (FRetirada.HasData);
end;

function TValor.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FOriginal <> '' then
      Result.AddPair('original', TJSONString.Create(FOriginal));
   if (FModalidadeAlteracao = 0) or (FModalidadeAlteracao = 1) then
      Result.AddPair('modalidadeAlteracao', TJSONNumber.Create(FModalidadeAlteracao));
   if FRetirada.HasData then
      Result.AddPair('retirada', FRetirada.ToJSON);
end;

procedure TValor.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('original')) then
      FOriginal := AJSON.Get('original').JsonValue.Value;
   if Assigned(AJSON.Get('modalidadeAlteracao')) then
      FModalidadeAlteracao := StrToInt(AJSON.Get('modalidadeAlteracao').JsonValue.Value);
   if Assigned(AJSON.Get('retirada')) then
      FRetirada.FromJSON(TJSONObject(AJSON.Get('retirada').JsonValue));
end;

{ TInfoAdicional }

function TInfoAdicional.HasData: Boolean;
begin
   Result := (FNome <> '') or (FValor <> '');
end;

function TInfoAdicional.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FNome <> '' then
      Result.AddPair('nome', TJSONString.Create(FNome));
   if FValor <> '' then
      Result.AddPair('valor', TJSONString.Create(FValor));
end;

procedure TInfoAdicional.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('nome')) then
      FNome := AJSON.Get('nome').JsonValue.Value;
   if Assigned(AJSON.Get('valor')) then
      FValor := AJSON.Get('valor').JsonValue.Value;
end;

{ TBBPixCobRequest }

constructor TBBPixCobRequest.Create;
begin
   FCalendario := TCalendario.Create;
   FDevedor := TDevedor.Create;
   FLoc := TLoc.Create;
   FValor := TValor.Create;
   FInfoAdicionais := TObjectList<TInfoAdicional>.Create(True);
end;

destructor TBBPixCobRequest.Destroy;
begin
   FCalendario.Free;
   FDevedor.Free;
   FLoc.Free;
   FValor.Free;
   FInfoAdicionais.Free;
   inherited;
end;

function TBBPixCobRequest.ToJSON: TJSONObject;
var
   Arr: TJSONArray;
   Info: TInfoAdicional;
begin
   Result := TJSONObject.Create;
   if FCalendario.HasData then
      Result.AddPair('calendario', FCalendario.ToJSON);
   if FDevedor.HasData then
      Result.AddPair('devedor', FDevedor.ToJSON);
   if FLoc.HasData then
      Result.AddPair('loc', FLoc.ToJSON);
   if FValor.HasData then
      Result.AddPair('valor', FValor.ToJSON);
   if FChave <> '' then
      Result.AddPair('chave', TJSONString.Create(FChave));
   if FSolicitacaoPagador <> '' then
      Result.AddPair('solicitacaoPagador', TJSONString.Create(FSolicitacaoPagador));

   if FInfoAdicionais.Count > 0 then
   begin
      Arr := TJSONArray.Create;
      for Info in FInfoAdicionais do
      begin
         if Info.HasData then
            Arr.AddElement(Info.ToJSON);
      end;
      if Arr.Size > 0 then
         Result.AddPair('infoAdicionais', Arr)
      else
         Arr.Free;
   end;
end;

procedure TBBPixCobRequest.FromJSON(AJSON: TJSONObject);
var
   Arr: TJSONArray;
   I: Integer;
   Info: TInfoAdicional;
begin
   if Assigned(AJSON.Get('calendario')) then
      FCalendario.FromJSON(TJSONObject(AJSON.Get('calendario').JsonValue));
   if Assigned(AJSON.Get('devedor')) then
      FDevedor.FromJSON(TJSONObject(AJSON.Get('devedor').JsonValue));
   if Assigned(AJSON.Get('loc')) then
      FLoc.FromJSON(TJSONObject(AJSON.Get('loc').JsonValue));
   if Assigned(AJSON.Get('valor')) then
      FValor.FromJSON(TJSONObject(AJSON.Get('valor').JsonValue));
   if Assigned(AJSON.Get('chave')) then
      FChave := AJSON.Get('chave').JsonValue.Value;
   if Assigned(AJSON.Get('solicitacaoPagador')) then
      FSolicitacaoPagador := AJSON.Get('solicitacaoPagador').JsonValue.Value;

   if Assigned(AJSON.Get('infoAdicionais')) then
   begin
      Arr := AJSON.Get('infoAdicionais').JsonValue as TJSONArray;
      for I := 0 to Arr.Size - 1 do
      begin
         Info := TInfoAdicional.Create;
         Info.FromJSON(Arr.Get(I) as TJSONObject);
         if Info.HasData then
            FInfoAdicionais.Add(Info)
         else
            Info.Free;
      end;
   end;
end;

end.

