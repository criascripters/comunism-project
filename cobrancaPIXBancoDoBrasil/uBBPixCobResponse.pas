unit uBBPixCobResponse;

interface

uses
   System.SysUtils, System.Classes, DBXJSON, Generics.Collections;

type
   TCalendario = class
   private
      FCriacao: string;
      FExpiracao: Integer;
   public
      property Criacao: string read FCriacao write FCriacao;
      property Expiracao: Integer read FExpiracao write FExpiracao;
      function ToJSON: TJSONObject;
      procedure FromJSON(AJSON: TJSONObject);
      function HasData: Boolean;
   end;

   TDevedor = class
   private
      FCPF: string;
      FCNPJ: string;
      FNome: string;
   public
      property CPF: string read FCPF write FCPF;
      property CNPJ: string read FCNPJ write FCNPJ;
      property Nome: string read FNome write FNome;
      function ToJSON: TJSONObject;
      procedure FromJSON(AJSON: TJSONObject);
      function HasData: Boolean;
   end;

   TLoc = class
   private
      FId: Int64;
      FLocation: string;
      FTipoCob: string;
      FCriacao: string;
   public
      property Id: Int64 read FId write FId;
      property Location: string read FLocation write FLocation;
      property TipoCob: string read FTipoCob write FTipoCob;
      property Criacao: string read FCriacao write FCriacao;
      function ToJSON: TJSONObject;
      procedure FromJSON(AJSON: TJSONObject);
      function HasData: Boolean;
   end;

   TValorSaqueTroco = class
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
      FSaque: TValorSaqueTroco;
      FTroco: TValorSaqueTroco;
   public
      constructor Create;
      destructor Destroy; override;
      property Saque: TValorSaqueTroco read FSaque write FSaque;
      property Troco: TValorSaqueTroco read FTroco write FTroco;
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

   TBBPixCobResponse = class
   private
      FCalendario: TCalendario;
      FTxid: string;
      FRevisao: Integer;
      FDevedor: TDevedor;
      FLoc: TLoc;
      FLocation: string;
      FStatus: string;
      FPixCopiaECola: string;
      FValor: TValor;
      FChave: string;
      FSolicitacaoPagador: string;
      FInfoAdicionais: TObjectList<TInfoAdicional>;
   public
      constructor Create;
      destructor Destroy; override;
      property Calendario: TCalendario read FCalendario write FCalendario;
      property Txid: string read FTxid write FTxid;
      property Revisao: Integer read FRevisao write FRevisao;
      property Devedor: TDevedor read FDevedor write FDevedor;
      property Loc: TLoc read FLoc write FLoc;
      property Location: string read FLocation write FLocation;
      property Status: string read FStatus write FStatus;
      property PixCopiaECola: string read FPixCopiaECola write FPixCopiaECola;
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
   Result := (FCriacao <> '') or (FExpiracao > 0);
end;

function TCalendario.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   Result.AddPair('criacao', TJSONString.Create(FCriacao));
   Result.AddPair('expiracao', TJSONNumber.Create(FExpiracao));
end;

procedure TCalendario.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('criacao')) then
      FCriacao := AJSON.Get('criacao').JsonValue.Value;
   if Assigned(AJSON.Get('expiracao')) then
      FExpiracao := StrToInt(AJSON.Get('expiracao').JsonValue.Value);
end;

{ TDevedor }

function TDevedor.HasData: Boolean;
begin
   Result := (FCPF <> '') or (FCNPJ <> '') or (FNome <> '');
end;

function TDevedor.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FCPF <> '' then
      Result.AddPair('cpf', TJSONString.Create(FCPF));
   if FCNPJ <> '' then
      Result.AddPair('cnpj', TJSONString.Create(FCNPJ));
   if FNome <> '' then
      Result.AddPair('nome', TJSONString.Create(FNome));
end;

procedure TDevedor.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('cpf')) then
      FCPF := AJSON.Get('cpf').JsonValue.Value;
   if Assigned(AJSON.Get('cnpj')) then
      FCNPJ := AJSON.Get('cnpj').JsonValue.Value;
   if Assigned(AJSON.Get('nome')) then
      FNome := AJSON.Get('nome').JsonValue.Value;
end;

{ TLoc }

function TLoc.HasData: Boolean;
begin
   Result := (FId > 0) or (FLocation <> '') or (FTipoCob <> '') or (FCriacao <> '');
end;

function TLoc.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FId > 0 then
      Result.AddPair('id', TJSONNumber.Create(FId));
   if FLocation <> '' then
      Result.AddPair('location', TJSONString.Create(FLocation));
   if FTipoCob <> '' then
      Result.AddPair('tipoCob', TJSONString.Create(FTipoCob));
   if FCriacao <> '' then
      Result.AddPair('criacao', TJSONString.Create(FCriacao));
end;

procedure TLoc.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('id')) then
      FId := StrToInt64(AJSON.Get('id').JsonValue.Value);
   if Assigned(AJSON.Get('location')) then
      FLocation := AJSON.Get('location').JsonValue.Value;
   if Assigned(AJSON.Get('tipoCob')) then
      FTipoCob := AJSON.Get('tipoCob').JsonValue.Value;
   if Assigned(AJSON.Get('criacao')) then
      FCriacao := AJSON.Get('criacao').JsonValue.Value;
end;

{ TValorSaqueTroco }

function TValorSaqueTroco.HasData: Boolean;
begin
   Result := (FValor <> '') or (FModalidadeAgente <> '') or (PrestadorDoServicoDeSaque <> '');
end;

function TValorSaqueTroco.ToJSON: TJSONObject;
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

procedure TValorSaqueTroco.FromJSON(AJSON: TJSONObject);
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

function TRetirada.HasData: Boolean;
begin
   Result := FSaque.HasData or FTroco.HasData;
end;

constructor TRetirada.Create;
begin
   FSaque := TValorSaqueTroco.Create;
   FTroco := TValorSaqueTroco.Create;
end;

destructor TRetirada.Destroy;
begin
   FSaque.Free;
   FTroco.Free;
   inherited;
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

function TValor.HasData: Boolean;
begin
   Result := (FOriginal <> '') or FRetirada.HasData;
end;

constructor TValor.Create;
begin
   FRetirada := TRetirada.Create;
end;

destructor TValor.Destroy;
begin
   FRetirada.Free;
   inherited;
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

{ TBBPixCobResponse }

constructor TBBPixCobResponse.Create;
begin
   FCalendario := TCalendario.Create;
   FDevedor := TDevedor.Create;
   FLoc := TLoc.Create;
   FValor := TValor.Create;
   FInfoAdicionais := TObjectList<TInfoAdicional>.Create(True);
end;

destructor TBBPixCobResponse.Destroy;
begin
   FCalendario.Free;
   FDevedor.Free;
   FLoc.Free;
   FValor.Free;
   FInfoAdicionais.Free;
   inherited;
end;

function TBBPixCobResponse.ToJSON: TJSONObject;
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
   if FTxid <> '' then
      Result.AddPair('txid', TJSONString.Create(FTxid));
   if FRevisao > -1 then
      Result.AddPair('revisao', TJSONNumber.Create(FRevisao));
   if FLocation <> '' then
      Result.AddPair('location', TJSONString.Create(FLocation));
   if FStatus <> '' then
      Result.AddPair('status', TJSONString.Create(FStatus));
   if FPixCopiaECola <> '' then
      Result.AddPair('pixCopiaECola', TJSONString.Create(FPixCopiaECola));

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

procedure TBBPixCobResponse.FromJSON(AJSON: TJSONObject);
var
   Arr: TJSONArray;
   I: Integer;
   InfoObj: TInfoAdicional;
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
   if Assigned(AJSON.Get('txid')) then
      FTxid := AJSON.Get('txid').JsonValue.Value;
   if Assigned(AJSON.Get('revisao')) then
      FRevisao := StrToInt(AJSON.Get('revisao').JsonValue.Value);
   if Assigned(AJSON.Get('location')) then
      FLocation := AJSON.Get('location').JsonValue.Value;
   if Assigned(AJSON.Get('status')) then
      FStatus := AJSON.Get('status').JsonValue.Value;
   if Assigned(AJSON.Get('pixCopiaECola')) then
      FPixCopiaECola := AJSON.Get('pixCopiaECola').JsonValue.Value;

   if Assigned(AJSON.Get('infoAdicionais')) then
   begin
      Arr := TJSONArray(AJSON.Get('infoAdicionais').JsonValue);
      for I := 0 to Arr.Size - 1 do
      begin
         InfoObj := TInfoAdicional.Create;
         InfoObj.FromJSON(Arr.Get(I) as TJSONObject);
         if InfoObj.HasData then
            FInfoAdicionais.Add(InfoObj)
         else
            InfoObj.Free;
      end;
   end;
end;

end.

