unit uBBAuth;

interface

uses
   SysUtils, Classes, DBXJSON, uBBAuthRequest;

type
   TBBAuth = class
   private
      FAccessToken: string;
      FTokenType: string;
      FExpiresIn: Integer;
      FScope: string;
   public
      constructor Create(const AClientID, AClientSecret, AScope: string);
      property AccessToken: string read FAccessToken;
      property TokenType: string read FTokenType;
      property ExpiresIn: Integer read FExpiresIn;
      property Scope: string read FScope;
   end;

implementation

{ TBBAuth }

constructor TBBAuth.Create(const AClientID, AClientSecret, AScope: string);
var
   RawJson: string;
   Json: TJSONObject;
   Val: TJSONValue;
begin
   inherited Create;

   RawJson := GetBBAccessToken(AClientID, AClientSecret, AScope);

   if RawJson = '' then
      raise Exception.Create('Não foi possível obter o token do Banco do Brasil.');

   Json := TJSONObject.ParseJSONValue(RawJson) as TJSONObject;
   if not Assigned(Json) then
      raise Exception.Create('Resposta do BB inválida (não é JSON)');

   try
      Val := Json.Get('access_token').JsonValue;
      if Assigned(Val) then
         FAccessToken := Val.Value;

      Val := Json.Get('token_type').JsonValue;
      if Assigned(Val) then
         FTokenType := Val.Value;

      Val := Json.Get('expires_in').JsonValue;
      if Assigned(Val) then
         FExpiresIn := StrToIntDef(Val.Value, 0);

      Val := Json.Get('scope').JsonValue;
      if Assigned(Val) then
         FScope := Val.Value;
   finally
      Json.Free;
   end;
end;

end.
