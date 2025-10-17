unit uBBPixCobConsultaTxid;

interface

uses
   SysUtils, Classes, Generics.Collections, DBXJSON, IdHTTP, IdSSLOpenSSL;

type
   TCalendario = class
   private
      FCriacao: string;
      FExpiracao: Integer;
   public
      property Criacao: string read FCriacao write FCriacao;
      property Expiracao: Integer read FExpiracao write FExpiracao;
      procedure FromJSON(AJSON: TJSONObject);
      function ToJSON: TJSONObject;
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
      procedure FromJSON(AJSON: TJSONObject);
      function ToJSON: TJSONObject;
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
      procedure FromJSON(AJSON: TJSONObject);
      function ToJSON: TJSONObject;
      function HasData: Boolean;
   end;

   TValor = class
   private
      FOriginal: string;
   public
      property Original: string read FOriginal write FOriginal;
      procedure FromJSON(AJSON: TJSONObject);
      function ToJSON: TJSONObject;
      function HasData: Boolean;
   end;

   TInfoAdicional = class
   private
      FNome: string;
      FValor: string;
   public
      property Nome: string read FNome write FNome;
      property Valor: string read FValor write FValor;
      procedure FromJSON(AJSON: TJSONObject);
      function ToJSON: TJSONObject;
      function HasData: Boolean;
   end;

type
   TDevolucao = class
   private
      FId: string;
      FRtrId: string;
      FValor: string;
      FHorarioSolicitacao: string;
      FHorarioLiquidacao: string;
      FStatus: string;
      FMotivo: string;
   public
      property Id: string read FId write FId;
      property RtrId: string read FRtrId write FRtrId;
      property Valor: string read FValor write FValor;
      property HorarioSolicitacao: string read FHorarioSolicitacao write FHorarioSolicitacao;
      property HorarioLiquidacao: string read FHorarioLiquidacao write FHorarioLiquidacao;
      property Status: string read FStatus write FStatus;
      property Motivo: string read FMotivo write FMotivo;
      function ToJSON: TJSONObject;
      procedure FromJSON(AJSON: TJSONObject);
      function HasData: Boolean;
   end;

   TPix = class
   private
      FEndToEndId: string;
      FTxid: string;
      FValor: string;
      FHorario: string;
      FInfoPagador: string;
      FDevolucoes: TObjectList<TDevolucao>;
   public
      constructor Create;
      destructor Destroy; override;
      property EndToEndId: string read FEndToEndId write FEndToEndId;
      property Txid: string read FTxid write FTxid;
      property Valor: string read FValor write FValor;
      property Horario: string read FHorario write FHorario;
      property InfoPagador: string read FInfoPagador write FInfoPagador;
      property Devolucoes: TObjectList<TDevolucao> read FDevolucoes write FDevolucoes;
      procedure FromJSON(AJSON: TJSONObject);
      function ToJSON: TJSONObject;
      function HasData: Boolean;
   end;

   TBBPixCobConsultaTxidResponse = class
   private
      FCalendario: TCalendario;
      FTxid: string;
      FRevisao: Integer;
      FLoc: TLoc;
      FLocation: string;
      FStatus: string;
      FDevedor: TDevedor;
      FValor: TValor;
      FChave: string;
      FSolicitacaoPagador: string;
      FInfoAdicionais: TObjectList<TInfoAdicional>;
      FPix: TObjectList<TPix>;
   public
      constructor Create;
      destructor Destroy; override;
      property Calendario: TCalendario read FCalendario write FCalendario;
      property Txid: string read FTxid write FTxid;
      property Revisao: Integer read FRevisao write FRevisao;
      property Loc: TLoc read FLoc write FLoc;
      property Location: string read FLocation write FLocation;
      property Status: string read FStatus write FStatus;
      property Devedor: TDevedor read FDevedor write FDevedor;
      property Valor: TValor read FValor write FValor;
      property Chave: string read FChave write FChave;
      property SolicitacaoPagador: string read FSolicitacaoPagador write FSolicitacaoPagador;
      property InfoAdicionais: TObjectList<TInfoAdicional> read FInfoAdicionais write FInfoAdicionais;
      property Pix: TObjectList<TPix> read FPix write FPix;
      procedure FromJSON(AJSON: TJSONObject);
      function ToJSON: TJSONObject;
   end;

   TBBPixCobConsultaTxid = class
   public
      class function Consultar(const AAccessToken, ATxid, AAppKey, ACertFile, AKeyFile: string): TBBPixCobConsultaTxidResponse;
   end;

implementation

class function TBBPixCobConsultaTxid.Consultar(
  const AAccessToken, ATxid, AAppKey, ACertFile, AKeyFile: string): TBBPixCobConsultaTxidResponse;
var
   LHTTP: TIdHTTP;
   LSSL: TIdSSLIOHandlerSocketOpenSSL;
   LURL, LResponse: string;
   LJSON: TJSONObject;
begin
   Result := nil;

   LHTTP := TIdHTTP.Create(nil);
   LSSL := TIdSSLIOHandlerSocketOpenSSL.Create(nil);
   LSSL.SSLOptions.Method := sslvTLSv1_2;
   LSSL.SSLOptions.SSLVersions := [sslvTLSv1_2];
   LSSL.SSLOptions.CertFile := ACertFile;
   LSSL.SSLOptions.KeyFile := AKeyFile;
   LSSL.PassThrough := False;
   try
      LHTTP.IOHandler := LSSL;
      LHTTP.Request.CustomHeaders.Clear;
      LHTTP.Request.CustomHeaders.AddValue('Authorization', 'Bearer ' + AAccessToken);
      LHTTP.Request.CustomHeaders.AddValue('Accept', 'application/json');

      LURL := Format('https://api-pix.bb.com.br/pix/v2/cob/%s?gw-dev-app-key=%s',
                     [ATxid, AAppKey]);

      LJSON := nil;
      try
         try
            LResponse := LHTTP.Get(LURL);
            LJSON := TJSONObject.ParseJSONValue(LResponse) as TJSONObject;
            Result := TBBPixCobConsultaTxidResponse.Create;
            Result.FromJSON(LJSON);
         except
            on E: EIdHTTPProtocolException do
               raise Exception.Create(E.ErrorMessage);
         end;
      finally
         LJSON.Free;
      end;
   finally
      LHTTP.Free;
      LSSL.Free;
   end;
end;

{ TCalendario }

function TCalendario.HasData: Boolean;
begin
   Result := (FCriacao <> '') or (FExpiracao > 0);
end;

procedure TCalendario.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('criacao')) then
      FCriacao := AJSON.Get('criacao').JsonValue.Value;
   if Assigned(AJSON.Get('expiracao')) then
      FExpiracao := StrToInt(AJSON.Get('expiracao').JsonValue.Value);
end;

function TCalendario.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FCriacao <> '' then
      Result.AddPair('criacao', TJSONString.Create(FCriacao));
   if FExpiracao > 0 then
      Result.AddPair('expiracao', TJSONNumber.Create(FExpiracao));
end;

{ TLoc }

function TLoc.HasData: Boolean;
begin
   Result := (FId > 0) or (FLocation <> '') or (FTipoCob <> '') or (FCriacao <> '');
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

{ TDevedor }

function TDevedor.HasData: Boolean;
begin
   Result := (FCpf <> '') or (FCnpj <> '') or (FNome <> '');
end;

procedure TDevedor.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('cpf')) then
      Fcpf := AJSON.Get('cpf').JsonValue.Value;
   if Assigned(AJSON.Get('cnpj')) then
      FCnpj := AJSON.Get('cnpj').JsonValue.Value;
   if Assigned(AJSON.Get('nome')) then
      FNome := AJSON.Get('nome').JsonValue.Value;
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

{ TValor }

function TValor.HasData: Boolean;
begin
   Result := FOriginal <> '';
end;

procedure TValor.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('original')) then
      FOriginal := AJSON.Get('original').JsonValue.Value;
end;

function TValor.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FOriginal <> '' then
      Result.AddPair('original', TJSONString.Create(FOriginal));
end;

{ TInfoAdicional }

function TInfoAdicional.HasData: Boolean;
begin
   Result := (FNome <> '') or (FValor <> '');
end;

procedure TInfoAdicional.FromJSON(AJSON: TJSONObject);
begin
   if Assigned(AJSON.Get('nome')) then
      FNome := AJSON.Get('nome').JsonValue.Value;
   if Assigned(AJSON.Get('valor')) then
      FValor := AJSON.Get('valor').JsonValue.Value;
end;

function TInfoAdicional.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FNome <> '' then
      Result.AddPair('nome', TJSONString.Create(FNome));
   if FValor <> '' then
      Result.AddPair('valor', TJSONString.Create(FValor));
end;

{ TDevolucao}

function TDevolucao.HasData: Boolean;
begin
   Result := (FId <> '') or (FRtrId <> '') or (FValor <> '') or (FHorarioSolicitacao <> '') or (FHorarioLiquidacao <> '') or (FStatus <> '') or (FMotivo <> '');
end;

function TDevolucao.ToJSON: TJSONObject;
var
   LHorario: TJSONObject;
begin
   Result := TJSONObject.Create;
   Result.AddPair('id', FId);
   Result.AddPair('rtrId', FRtrId);
   Result.AddPair('valor', FValor);

   LHorario := TJSONObject.Create;
   if FHorarioSolicitacao <> '' then
      LHorario.AddPair('solicitacao', FHorarioSolicitacao);
   if FHorarioLiquidacao <> '' then
      LHorario.AddPair('liquidacao', FHorarioLiquidacao);
   Result.AddPair('horario', LHorario);

   Result.AddPair('status', FStatus);
   if FMotivo <> '' then
      Result.AddPair('motivo', FMotivo);
end;

procedure TDevolucao.FromJSON(AJSON: TJSONObject);
var
   LHorario: TJSONObject;
begin
   if Assigned(AJSON.Get('id')) then
      FId := AJSON.Get('id').JsonValue.Value;
   if Assigned(AJSON.Get('rtrId')) then
      FRtrId := AJSON.Get('rtrId').JsonValue.Value;
   if Assigned(AJSON.Get('valor')) then
      FValor := AJSON.Get('valor').JsonValue.Value;

   if Assigned(AJSON.Get('horario')) then
   begin
      LHorario := AJSON.Get('horario').JsonValue as TJSONObject;
      if Assigned(LHorario.Get('solicitacao')) then
         FHorarioSolicitacao := LHorario.Get('solicitacao').JsonValue.Value;
      if Assigned(LHorario.Get('liquidacao')) then
         FHorarioLiquidacao := LHorario.Get('liquidacao').JsonValue.Value;
   end;

   if Assigned(AJSON.Get('status')) then
      FStatus := AJSON.Get('status').JsonValue.Value;
   if Assigned(AJSON.Get('motivo')) then
      FMotivo := AJSON.Get('motivo').JsonValue.Value;
end;

{ TPix }

function TPix.HasData: Boolean;
begin
   Result := (FEndToEndId <> '') or (FTxid <> '') or (FValor <> '') or (FHorario <> '') or (FInfoPagador <> '');
end;

constructor TPix.Create;
begin
   FDevolucoes := TObjectList<TDevolucao>.Create(True);
end;

destructor TPix.Destroy;
begin
   FDevolucoes.Free;
   inherited;
end;

procedure TPix.FromJSON(AJSON: TJSONObject);
var
   JArr: TJSONArray;
   I: Integer;
   D: TDevolucao;
begin
   if Assigned(AJSON.Get('endToEndId')) then
      FEndToEndId := AJSON.Get('endToEndId').JsonValue.Value;
   if Assigned(AJSON.Get('txid')) then
      FTxid := AJSON.Get('txid').JsonValue.Value;
   if Assigned(AJSON.Get('valor')) then
      FValor := AJSON.Get('valor').JsonValue.Value;
   if Assigned(AJSON.Get('horario')) then
      FHorario := AJSON.Get('horario').JsonValue.Value;
   if Assigned(AJSON.Get('infoPagador')) then
      FInfoPagador := AJSON.Get('infoPagador').JsonValue.Value;

   if Assigned(AJSON.Get('devolucoes')) then
   begin
      JArr := AJSON.Get('devolucoes').JsonValue as TJSONArray;
      for I := 0 to JArr.Size - 1 do
      begin
         D := TDevolucao.Create;
         D.FromJSON(JArr.Get(I) as TJSONObject);
         FDevolucoes.Add(D);
      end;
   end;
end;

function TPix.ToJSON: TJSONObject;
var
   Arr: TJSONArray;
   I: Integer;
begin
   Result := TJSONObject.Create;
   if FEndToEndId <> '' then
      Result.AddPair('endToEndId', TJSONString.Create(FEndToEndId));
   if FTxid <> '' then
      Result.AddPair('txid', TJSONString.Create(FTxid));
   if FValor <> '' then
      Result.AddPair('valor', TJSONString.Create(FValor));
   if FHorario <> '' then
      Result.AddPair('horario', TJSONString.Create(FHorario));
   if FInfoPagador <> '' then
      Result.AddPair('infoPagador', TJSONString.Create(FInfoPagador));

   if FDevolucoes.Count > 0 then
   begin
      Arr := TJSONArray.Create;
      for I := 0 to FDevolucoes.Count - 1 do
      begin
         if FDevolucoes[I].HasData then
            Arr.AddElement(FDevolucoes[I].ToJSON);
      end;
      if Arr.Size > 0 then
         Result.AddPair('devolucoes', Arr)
      else
         Arr.Free;
   end;
end;

{ TBBPixCobConsultaTxidResponse }

constructor TBBPixCobConsultaTxidResponse.Create;
begin
   FCalendario := TCalendario.Create;
   FDevedor := TDevedor.Create;
   FLoc := TLoc.Create;
   FValor := TValor.Create;
   FInfoAdicionais := TObjectList<TInfoAdicional>.Create(True);
   FPix := TObjectList<TPix>.Create(True);
end;

destructor TBBPixCobConsultaTxidResponse.Destroy;
begin
   FCalendario.Free;
   FDevedor.Free;
   FLoc.Free;
   FValor.Free;
   FInfoAdicionais.Free;
   FPix.Free;
   inherited;
end;

procedure TBBPixCobConsultaTxidResponse.FromJSON(AJSON: TJSONObject);
var
   Arr1, Arr2: TJSONArray;
   I: Integer;
   InfoObj: TInfoAdicional;
   PixObj: TPix;
begin
   if Assigned(AJSON.Get('calendario')) then
      FCalendario.FromJSON(TJSONObject(AJSON.Get('calendario').JsonValue));
   if Assigned(AJSON.Get('txid')) then
      FTxid := AJSON.Get('txid').JsonValue.Value;
   if Assigned(AJSON.Get('revisao')) then
      FRevisao := StrToInt(AJSON.Get('revisao').JsonValue.Value);
   if Assigned(AJSON.Get('loc')) then
      FLoc.FromJSON(TJSONObject(AJSON.Get('loc').JsonValue));
   if Assigned(AJSON.Get('location')) then
      FLocation := AJSON.Get('location').JsonValue.Value;
   if Assigned(AJSON.Get('status')) then
      FStatus := AJSON.Get('status').JsonValue.Value;
   if Assigned(AJSON.Get('devedor')) then
      FDevedor.FromJSON(TJSONObject(AJSON.Get('devedor').JsonValue));
   if Assigned(AJSON.Get('valor')) then
      FValor.FromJSON(TJSONObject(AJSON.Get('valor').JsonValue));
   if Assigned(AJSON.Get('chave')) then
      FChave := AJSON.Get('chave').JsonValue.Value;
   if Assigned(AJSON.Get('solicitacaoPagador')) then
      FSolicitacaoPagador := AJSON.Get('solicitacaoPagador').JsonValue.Value;

   if Assigned(AJSON.Get('infoAdicionais')) then
   begin
      Arr1 := TJSONArray(AJSON.Get('infoAdicionais').JsonValue);
      for I := 0 to Arr1.Size - 1 do
      begin
         InfoObj := TInfoAdicional.Create;
         InfoObj.FromJSON(Arr1.Get(I) as TJSONObject);
         if InfoObj.HasData then
            FInfoAdicionais.Add(InfoObj)
         else
            InfoObj.Free;
      end;
   end;

   if Assigned(AJSON.Get('pix')) then
   begin
      Arr2 := TJSONArray(AJSON.Get('pix').JsonValue);
      for I := 0 to Arr2.Size - 1 do
      begin
         PixObj := TPix.Create;
         PixObj.FromJSON(Arr2.Get(I) as TJSONObject);
         if PixObj.HasData then
            FPix.Add(PixObj)
         else
            PixObj.Free;
      end;
   end;
end;

function TBBPixCobConsultaTxidResponse.ToJSON: TJSONObject;
var
   Arr1, Arr2: TJSONArray;
   Info: TInfoAdicional;
   Pix: TPix;
begin
   Result := TJSONObject.Create;
   if FCalendario.HasData then
      Result.AddPair('calendario', FCalendario.ToJSON);
   if FTxid <> '' then
      Result.AddPair('txid', TJSONString.Create(FTxid));
   if FRevisao > -1 then
      Result.AddPair('revisao', TJSONNumber.Create(FRevisao));
   if FLoc.HasData then
      Result.AddPair('loc', FLoc.ToJSON);
   if FLocation <> '' then
      Result.AddPair('location', TJSONString.Create(FLocation));
   if FStatus <> '' then
      Result.AddPair('status', TJSONString.Create(FStatus));
   if FDevedor.HasData then
      Result.AddPair('devedor', FDevedor.ToJSON);
   if FValor.HasData then
      Result.AddPair('valor', FValor.ToJSON);
   if FChave <> '' then
      Result.AddPair('chave', TJSONString.Create(FChave));
   if FSolicitacaoPagador <> '' then
      Result.AddPair('solicitacaoPagador', TJSONString.Create(FSolicitacaoPagador));

   if FInfoAdicionais.Count > 0 then
   begin
      Arr1 := TJSONArray.Create;
      for Info in FInfoAdicionais do
      begin
         if Info.HasData then
            Arr1.AddElement(Info.ToJSON);
      end;
      if Arr1.Size > 0 then
         Result.AddPair('infoAdicionais', Arr1)
      else
         Arr1.Free;
   end;

   if FPix.Count > 0 then
   begin
      Arr2 := TJSONArray.Create;
      for Pix in FPix do
      begin
         if Pix.HasData then
            Arr2.AddElement(Pix.ToJSON);
      end;
      if Arr2.Size > 0 then
         Result.AddPair('pix', Arr2)
      else
         Arr2.Free;
   end;
end;

end.

