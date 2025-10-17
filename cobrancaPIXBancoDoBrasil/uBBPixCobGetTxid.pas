unit uBBPixCobGetTxid;

interface

uses
   System.SysUtils, System.Classes, DBXJSON, Generics.Collections;

type
   THorarioDevolucao = class
   private
      FSolicitacao: string;
      FLiquidacao: string;
   public
      property Solicitacao: string read FSolicitacao write FSolicitacao;
      property Liquidacao: string read FLiquidacao write FLiquidacao;
      function ToJSON: TJSONObject;
      class function FromJSON(AJSON: TJSONObject): THorarioDevolucao;
   end;

   TDevolucao = class
   private
      FId: string;
      FRtrId: string;
      FValor: string;
      FHorario: THorarioDevolucao;
      FStatus: string;
      FMotivo: string;
   public
      property Id: string read FId write FId;
      property RtrId: string read FRtrId write FRtrId;
      property Valor: string read FValor write FValor;
      property Horario: THorarioDevolucao read FHorario write FHorario;
      property Status: string read FStatus write FStatus;
      property Motivo: string read FMotivo write FMotivo;

      function ToJSON: TJSONObject;
      class function FromJSON(AJSON: TJSONObject): TDevolucao;
      destructor Destroy; override;
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
      property EndToEndId: string read FEndToEndId write FEndToEndId;
      property Txid: string read FTxid write FTxid;
      property Valor: string read FValor write FValor;
      property Horario: string read FHorario write FHorario;
      property InfoPagador: string read FInfoPagador write FInfoPagador;
      property Devolucoes: TObjectList<TDevolucao> read FDevolucoes write FDevolucoes;

      constructor Create;
      destructor Destroy; override;
      function ToJSON: TJSONObject;
      class function FromJSON(AJSON: TJSONObject): TPix;
   end;

   TCalendario = class
   private
      FCriacao: string;
      FExpiracao: Integer;
   public
      property Criacao: string read FCriacao write FCriacao;
      property Expiracao: Integer read FExpiracao write FExpiracao;
      function ToJSON: TJSONObject;
      class function FromJSON(AJSON: TJSONObject): TCalendario;
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
      class function FromJSON(AJSON: TJSONObject): TDevedor;
   end;

   TLoc = class
   private
      FId: string;
      FLocation: string;
      FTipoCob: string;
      FCriacao: string;
   public
      property Id: string read FId write FId;
      property Location: string read FLocation write FLocation;
      property TipoCob: string read FTipoCob write FTipoCob;
      property Criacao: string read FCriacao write FCriacao;
      function ToJSON: TJSONObject;
      class function FromJSON(AJSON: TJSONObject): TLoc;
   end;

   TValor = class
   private
      FOriginal: string;
   public
      property Original: string read FOriginal write FOriginal;
      function ToJSON: TJSONObject;
      class function FromJSON(AJSON: TJSONObject): TValor;
   end;

   TInfoAdicional = class
   private
      FNome: string;
      FValor: string;
   public
      property Nome: string read FNome write FNome;
      property Valor: string read FValor write FValor;
      function ToJSON: TJSONObject;
      class function FromJSON(AJSON: TJSONObject): TInfoAdicional;
   end;

   TBBPixCobGetTxid = class
   private
      FCalendario: TCalendario;
      FDevedor: TDevedor;
      FLoc: TLoc;
      FValor: TValor;
      FChave: string;
      FSolicitacaoPagador: string;
      FInfoAdicionais: TObjectList<TInfoAdicional>;
      FTxid: string;
      FRevisao: Integer;
      FLocation: string;
      FStatus: string;
      FPix: TObjectList<TPix>;
   public
      property Calendario: TCalendario read FCalendario write FCalendario;
      property Devedor: TDevedor read FDevedor write FDevedor;
      property Loc: TLoc read FLoc write FLoc;
      property Valor: TValor read FValor write FValor;
      property Chave: string read FChave write FChave;
      property SolicitacaoPagador: string read FSolicitacaoPagador write FSolicitacaoPagador;
      property InfoAdicionais: TObjectList<TInfoAdicional> read FInfoAdicionais write FInfoAdicionais;
      property Txid: string read FTxid write FTxid;
      property Revisao: Integer read FRevisao write FRevisao;
      property Location: string read FLocation write FLocation;
      property Status: string read FStatus write FStatus;
      property Pix: TObjectList<TPix> read FPix write FPix;

      constructor Create;
      destructor Destroy; override;
      function ToJSON: TJSONObject;
      class function FromJSON(AJSON: TJSONObject): TBBPixCobGetTxid;
   end;

implementation

uses
   DBXJSONReflect;

{ THorarioDevolucao }

function THorarioDevolucao.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FSolicitacao <> '' then Result.AddPair('solicitacao', FSolicitacao);
   if FLiquidacao <> '' then Result.AddPair('liquidacao', FLiquidacao);
end;

class function THorarioDevolucao.FromJSON(AJSON: TJSONObject): THorarioDevolucao;
var
   V: TJSONPair;
begin
   Result := THorarioDevolucao.Create;
   V := AJSON.Get('solicitacao'); if Assigned(V) then Result.FSolicitacao := V.JsonValue.Value;
   V := AJSON.Get('liquidacao'); if Assigned(V) then Result.FLiquidacao := V.JsonValue.Value;
end;

{ TDevolucao }

destructor TDevolucao.Destroy;
begin
   FHorario.Free;
   inherited;
end;

function TDevolucao.ToJSON: TJSONObject;
var
   Arr: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FId <> '' then Result.AddPair('id', FId);
   if FRtrId <> '' then Result.AddPair('rtrId', FRtrId);
   if FValor <> '' then Result.AddPair('valor', FValor);
   if Assigned(FHorario) then Result.AddPair('horario', FHorario.ToJSON);
   if FStatus <> '' then Result.AddPair('status', FStatus);
   if FMotivo <> '' then Result.AddPair('motivo', FMotivo);
end;

class function TDevolucao.FromJSON(AJSON: TJSONObject): TDevolucao;
var
   V: TJSONPair;
begin
   Result := TDevolucao.Create;
   V := AJSON.Get('id'); if Assigned(V) then Result.FId := V.JsonValue.Value;
   V := AJSON.Get('rtrId'); if Assigned(V) then Result.FRtrId := V.JsonValue.Value;
   V := AJSON.Get('valor'); if Assigned(V) then Result.FValor := V.JsonValue.Value;
   V := AJSON.Get('horario'); if Assigned(V) and (V.JsonValue is TJSONObject) then Result.FHorario := THorarioDevolucao.FromJSON(TJSONObject(V.JsonValue));
   V := AJSON.Get('status'); if Assigned(V) then Result.FStatus := V.JsonValue.Value;
   V := AJSON.Get('motivo'); if Assigned(V) then Result.FMotivo := V.JsonValue.Value;
end;

{ TPix }

constructor TPix.Create;
begin
   FDevolucoes := TObjectList<TDevolucao>.Create;
end;

destructor TPix.Destroy;
begin
   FDevolucoes.Free;
   inherited;
end;

function TPix.ToJSON: TJSONObject;
var
   Arr: TJSONArray;
   D: TDevolucao;
begin
   Result := TJSONObject.Create;
   if FEndToEndId <> '' then Result.AddPair('endToEndId', FEndToEndId);
   if FTxid <> '' then Result.AddPair('txid', FTxid);
   if FValor <> '' then Result.AddPair('valor', FValor);
   if FHorario <> '' then Result.AddPair('horario', FHorario);
   if FInfoPagador <> '' then Result.AddPair('infoPagador', FInfoPagador);
   if FDevolucoes.Count > 0 then
   begin
      Arr := TJSONArray.Create;
      for D in FDevolucoes do
         Arr.AddElement(D.ToJSON);
      Result.AddPair('devolucoes', Arr);
   end;
end;

class function TPix.FromJSON(AJSON: TJSONObject): TPix;
var
   V: TJSONPair;
   Arr: TJSONArray;
   I: Integer;
begin
   Result := TPix.Create;
   V := AJSON.Get('endToEndId'); if Assigned(V) then Result.FEndToEndId := V.JsonValue.Value;
   V := AJSON.Get('txid'); if Assigned(V) then Result.FTxid := V.JsonValue.Value;
   V := AJSON.Get('valor'); if Assigned(V) then Result.FValor := V.JsonValue.Value;
   V := AJSON.Get('horario'); if Assigned(V) then Result.FHorario := V.JsonValue.Value;
   V := AJSON.Get('infoPagador'); if Assigned(V) then Result.FInfoPagador := V.JsonValue.Value;
   V := AJSON.Get('devolucoes');
   if Assigned(V) and (V.JsonValue is TJSONArray) then
   begin
      Arr := TJSONArray(V.JsonValue);
      for I := 0 to Arr.Size - 1 do
         Result.FDevolucoes.Add(TDevolucao.FromJSON(Arr.Get(I) as TJSONObject));
   end;
end;

{ TCalendario }

function TCalendario.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FCriacao <> '' then Result.AddPair('criacao', FCriacao);
   if FExpiracao > 0 then Result.AddPair('expiracao', TJSONNumber.Create(FExpiracao));
end;

class function TCalendario.FromJSON(AJSON: TJSONObject): TCalendario;
var
   V: TJSONPair;
begin
   Result := TCalendario.Create;
   V := AJSON.Get('criacao'); if Assigned(V) then Result.FCriacao := V.JsonValue.Value;
   V := AJSON.Get('expiracao'); if Assigned(V) then Result.FExpiracao := StrToIntDef(V.JsonValue.Value, 0);
end;

{ TDevedor }

function TDevedor.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FCPF <> '' then Result.AddPair('cpf', FCPF);
   if FCNPJ <> '' then Result.AddPair('cnpj', FCNPJ);
   if FNome <> '' then Result.AddPair('nome', FNome);
end;

class function TDevedor.FromJSON(AJSON: TJSONObject): TDevedor;
var
   V: TJSONPair;
begin
   Result := TDevedor.Create;
   V := AJSON.Get('cpf'); if Assigned(V) then Result.FCPF := V.JsonValue.Value;
   V := AJSON.Get('cnpj'); if Assigned(V) then Result.FCNPJ := V.JsonValue.Value;
   V := AJSON.Get('nome'); if Assigned(V) then Result.FNome := V.JsonValue.Value;
end;

{ TLoc }

function TLoc.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FId <> '' then Result.AddPair('id', FId);
   if FLocation <> '' then Result.AddPair('location', FLocation);
   if FTipoCob <> '' then Result.AddPair('tipoCob', FTipoCob);
   if FCriacao <> '' then Result.AddPair('criacao', FCriacao);
end;

class function TLoc.FromJSON(AJSON: TJSONObject): TLoc;
var
  V: TJSONPair;
begin
   Result := TLoc.Create;
   V := AJSON.Get('id'); if Assigned(V) then Result.FId := V.JsonValue.Value;
   V := AJSON.Get('location'); if Assigned(V) then Result.FLocation := V.JsonValue.Value;
   V := AJSON.Get('tipoCob'); if Assigned(V) then Result.FTipoCob := V.JsonValue.Value;
   V := AJSON.Get('criacao'); if Assigned(V) then Result.FCriacao := V.JsonValue.Value;
end;

{ TValor }

function TValor.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FOriginal <> '' then Result.AddPair('original', FOriginal);
end;

class function TValor.FromJSON(AJSON: TJSONObject): TValor;
var
   V: TJSONPair;
begin
   Result := TValor.Create;
   V := AJSON.Get('original'); if Assigned(V) then Result.FOriginal := V.JsonValue.Value;
end;

{ TInfoAdicional }

function TInfoAdicional.ToJSON: TJSONObject;
begin
   Result := TJSONObject.Create;
   if FNome <> '' then Result.AddPair('nome', FNome);
   if FValor <> '' then Result.AddPair('valor', FValor);
end;

class function TInfoAdicional.FromJSON(AJSON: TJSONObject): TInfoAdicional;
var
   V: TJSONPair;
begin
   Result := TInfoAdicional.Create;
   V := AJSON.Get('nome'); if Assigned(V) then Result.FNome := V.JsonValue.Value;
   V := AJSON.Get('valor'); if Assigned(V) then Result.FValor := V.JsonValue.Value;
end;

{ TBBPixCobGetTxid }

constructor TBBPixCobGetTxid.Create;
begin
   FCalendario := TCalendario.Create;
   FDevedor := TDevedor.Create;
   FLoc := TLoc.Create;
   FValor := TValor.Create;
   FInfoAdicionais := TObjectList<TInfoAdicional>.Create;
   FPix := TObjectList<TPix>.Create;
end;

destructor TBBPixCobGetTxid.Destroy;
var
   Info: TInfoAdicional;
   P: TPix;
begin
   FCalendario.Free;
   FDevedor.Free;
   FLoc.Free;
   FValor.Free;
   for Info in FInfoAdicionais do Info.Free;
   FInfoAdicionais.Free;
   for P in FPix do P.Free;
   FPix.Free;
   inherited;
end;

function TBBPixCobGetTxid.ToJSON: TJSONObject;
var
   Arr: TJSONArray;
   Info: TInfoAdicional;
   P: TPix;
begin
   Result := TJSONObject.Create;
   if Assigned(FCalendario) then Result.AddPair('calendario', FCalendario.ToJSON);
   if Assigned(FDevedor) then Result.AddPair('devedor', FDevedor.ToJSON);
   if Assigned(FLoc) then Result.AddPair('loc', FLoc.ToJSON);
   if Assigned(FValor) then Result.AddPair('valor', FValor.ToJSON);
   if FChave <> '' then Result.AddPair('chave', FChave);
   if FTxid <> '' then Result.AddPair('txid', FTxid);
   if FRevisao > 0 then Result.AddPair('revisao', TJSONNumber.Create(FRevisao));
   if FLocation <> '' then Result.AddPair('location', FLocation);
   if FStatus <> '' then Result.AddPair('status', FStatus);
   if FSolicitacaoPagador <> '' then Result.AddPair('solicitacaoPagador', FSolicitacaoPagador);

   if FInfoAdicionais.Count > 0 then
   begin
      Arr := TJSONArray.Create;
      for Info in FInfoAdicionais do
         Arr.AddElement(Info.ToJSON);
      Result.AddPair('infoAdicionais', Arr);
   end;

   if FPix.Count > 0 then
   begin
      Arr := TJSONArray.Create;
      for P in FPix do
         Arr.AddElement(P.ToJSON);
      Result.AddPair('pix', Arr);
   end;
end;

class function TBBPixCobGetTxid.FromJSON(AJSON: TJSONObject): TBBPixCobGetTxid;
var
   V: TJSONPair;
   Arr: TJSONArray;
   I: Integer;
   Info: TInfoAdicional;
   P: TPix;
begin
   Result := TBBPixCobGetTxid.Create;

   V := AJSON.Get('calendario');
   if Assigned(V) and (V.JsonValue is TJSONObject) then
      Result.FCalendario := TCalendario.FromJSON(TJSONObject(V.JsonValue));

   V := AJSON.Get('devedor');
   if Assigned(V) and (V.JsonValue is TJSONObject) then
      Result.FDevedor := TDevedor.FromJSON(TJSONObject(V.JsonValue));

   V := AJSON.Get('loc');
   if Assigned(V) and (V.JsonValue is TJSONObject) then
      Result.FLoc := TLoc.FromJSON(TJSONObject(V.JsonValue));

   V := AJSON.Get('valor');
   if Assigned(V) and (V.JsonValue is TJSONObject) then
      Result.FValor := TValor.FromJSON(TJSONObject(V.JsonValue));

   V := AJSON.Get('chave'); if Assigned(V) then Result.FChave := V.JsonValue.Value;
   V := AJSON.Get('txid'); if Assigned(V) then Result.FTxid := V.JsonValue.Value;
   V := AJSON.Get('revisao'); if Assigned(V) then Result.FRevisao := StrToIntDef(V.JsonValue.Value, 0);
   V := AJSON.Get('location'); if Assigned(V) then Result.FLocation := V.JsonValue.Value;
   V := AJSON.Get('status'); if Assigned(V) then Result.FStatus := V.JsonValue.Value;
   V := AJSON.Get('solicitacaoPagador'); if Assigned(V) then Result.FSolicitacaoPagador := V.JsonValue.Value;

   V := AJSON.Get('infoAdicionais');
   if Assigned(V) and (V.JsonValue is TJSONArray) then
   begin
      Arr := TJSONArray(V.JsonValue);
      for I := 0 to Arr.Size - 1 do
      begin
         Info := TInfoAdicional.FromJSON(Arr.Get(I) as TJSONObject);
         Result.FInfoAdicionais.Add(Info);
      end;
   end;

   V := AJSON.Get('pix');
   if Assigned(V) and (V.JsonValue is TJSONArray) then
   begin
      Arr := TJSONArray(V.JsonValue);
      for I := 0 to Arr.Size - 1 do
      begin
         P := TPix.FromJSON(Arr.Get(I) as TJSONObject);
         Result.FPix.Add(P);
      end;
   end;
end;

end.

