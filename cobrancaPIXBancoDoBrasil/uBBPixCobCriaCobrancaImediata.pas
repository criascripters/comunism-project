unit uBBPixCobCriaCobrancaImediata;

interface

uses
   SysUtils, Classes, DBXJSON, IdHTTP, IdSSLOpenSSL,
   uBBPixCobRequest, uBBPixCobResponse;

type
   TBBPixCobCriaCobrancaImediata = class
   public
      class function CriarCobranca(const ARequest: TBBPixCobRequest;
        const AAccessToken, AAppKey, ACertFile, AKeyFile: string): TBBPixCobResponse;
   end;

implementation

{ TBBPixCobCriaCobrancaImediata }

class function TBBPixCobCriaCobrancaImediata.CriarCobranca(
  const ARequest: TBBPixCobRequest; const AAccessToken, AAppKey, ACertFile, AKeyFile: string): TBBPixCobResponse;
var
   Http: TIdHTTP;
   SSL: TIdSSLIOHandlerSocketOpenSSL;
   LURL: string;
   LBody: TStringStream;
   LResp: string;
   LJSON: TJSONObject;
begin
   Result := TBBPixCobResponse.Create;

   Http := TIdHTTP.Create(nil);
   SSL := TIdSSLIOHandlerSocketOpenSSL.Create(nil);
   SSL.SSLOptions.Method := sslvTLSv1_2;
   SSL.SSLOptions.SSLVersions := [sslvTLSv1_2];
   SSL.SSLOptions.CertFile := ACertFile;
   SSL.SSLOptions.KeyFile := AKeyFile;
   SSL.PassThrough := False;
   try
      Http.IOHandler := SSL;
      Http.Request.ContentType := 'application/json';
      Http.Request.CustomHeaders.Values['Authorization'] := 'Bearer ' + AAccessToken;

      LURL := 'https://api-pix.bb.com.br/pix/v2/cob?gw-dev-app-key=' + AAppKey;

      LBody := TStringStream.Create(ARequest.ToJSON.ToString, TEncoding.UTF8);
      LJSON := nil;
      try
         try
            LResp := Http.Post(LURL, LBody);
            LJSON := TJSONObject.ParseJSONValue(LResp) as TJSONObject;
            if Assigned(LJSON) then
               Result.FromJSON(LJSON);
         except
            on E: EIdHTTPProtocolException do
               raise Exception.Create(E.ErrorMessage);
         end;
      finally
         LJSON.Free;
         LBody.Free;
      end;
   finally
      SSL.Free;
      Http.Free;
   end;
end;

end.

