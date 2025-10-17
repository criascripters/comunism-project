unit uBBAuthRequest;

interface

uses
   Windows, SysUtils, Classes, WinInet, DBXJSON;

function GetBBAccessToken(const AClientID, AClientSecret: string;
   const AScope: string = 'cobrancas.boletos-info cobrancas.boletos-requisicao'): string;

implementation

function B64Encode(const S: string): string;
const
   CTable: AnsiString = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/';
var
   B: TBytes;
   I, P, Rem: Integer;
   B0, B1, B2: Integer;
begin
   Result := '';
   B := TEncoding.UTF8.GetBytes(S);
   if Length(B) = 0 then Exit;

   SetLength(Result, ((Length(B) + 2) div 3) * 4);
   P := 1;
   I := 0;

   while I + 2 < Length(B) do
   begin
      B0 := B[I];
      B1 := B[I + 1];
      B2 := B[I + 2];

      Result[P]     := Char(CTable[(B0 shr 2) + 1]);
      Result[P + 1] := Char(CTable[(((B0 and $03) shl 4) or (B1 shr 4)) + 1]);
      Result[P + 2] := Char(CTable[(((B1 and $0F) shl 2) or (B2 shr 6)) + 1]);
      Result[P + 3] := Char(CTable[(B2 and $3F) + 1]);

      Inc(I, 3);
      Inc(P, 4);
   end;

   Rem := Length(B) - I;
   if Rem = 1 then
   begin
      B0 := B[I];
      Result[P]     := Char(CTable[(B0 shr 2) + 1]);
      Result[P + 1] := Char(CTable[((B0 and $03) shl 4) + 1]);
      Result[P + 2] := '=';
      Result[P + 3] := '=';
   end
   else if Rem = 2 then
   begin
      B0 := B[I];
      B1 := B[I + 1];
      Result[P]     := Char(CTable[(B0 shr 2) + 1]);
      Result[P + 1] := Char(CTable[(((B0 and $03) shl 4) or (B1 shr 4)) + 1]);
      Result[P + 2] := Char(CTable[((B1 and $0F) shl 2) + 1]);
      Result[P + 3] := '=';
   end;
end;

function URLEncode(const AValue: string): string;
const
   HexMap: string = '0123456789ABCDEF';
var
   I: Integer;
   C: Byte;
   S: UTF8String;
begin
   Result := '';
   S := UTF8String(AValue);
   for I := 1 to Length(S) do
   begin
      C := Byte(S[I]);
      case Char(C) of
         'A'..'Z', 'a'..'z', '0'..'9', '-', '_', '.', '~':
            Result := Result + Char(C);
         ' ':
            Result := Result + '+'
      else
         Result := Result + '%' + HexMap[(C shr 4) + 1] + HexMap[(C and $0F) + 1];
      end;
   end;
end;

function GetBBAccessToken(const AClientID, AClientSecret: string;
   const AScope: string): string;
var
   hInet, hConnect, hRequest: HINTERNET;
   Headers: string;
   PostData: AnsiString;
   Buffer: array[0..1023] of AnsiChar;
   ReadSize: DWORD;
   Response: TStringStream;
   StatusCode: DWORD;
   StatusSize: DWORD;
   Index: DWORD;
   AuthBasic: string;
   SecFlags: DWORD;
begin
   Result := '';
   Response := TStringStream.Create('', TEncoding.UTF8);
   try
      // Força TLS 1.2
      SecFlags := $00000800; // SECURITY_FLAG_SECURE (TLS1.2)
      InternetSetOption(nil, 31 {INTERNET_OPTION_SECURITY_FLAGS}, @SecFlags, SizeOf(SecFlags));

      hInet := InternetOpen('DelphiXE4', INTERNET_OPEN_TYPE_PRECONFIG, nil, nil, 0);
      if hInet = nil then
         raise Exception.Create('InternetOpen falhou.');

      try
         hConnect := InternetConnect(hInet, PChar('oauth.bb.com.br'), INTERNET_DEFAULT_HTTPS_PORT,
            nil, nil, INTERNET_SERVICE_HTTP, 0, 0);
         if hConnect = nil then
            raise Exception.Create('InternetConnect falhou.');

         try
            hRequest := HttpOpenRequest(hConnect, 'POST', PChar('/oauth/token'), nil, nil, nil,
               INTERNET_FLAG_SECURE or INTERNET_FLAG_RELOAD or INTERNET_FLAG_NO_CACHE_WRITE, 0);
            if hRequest = nil then
               raise Exception.Create('HttpOpenRequest falhou.');

            try
               AuthBasic := 'Basic ' + B64Encode(AClientID + ':' + AClientSecret);

               Headers :=
                  'Content-Type: application/x-www-form-urlencoded'#13#10 +
                  'Authorization: ' + AuthBasic + #13#10;

               PostData :=
                  'grant_type=client_credentials' +
                  '&scope=' + AnsiString(URLEncode(AScope));

               if not HttpSendRequest(hRequest, PChar(Headers), Length(Headers),
                  PAnsiChar(PostData), Length(PostData)) then
                  raise Exception.Create('HttpSendRequest falhou.');

               // Lê HTTP Status Code
               StatusSize := SizeOf(StatusCode);
               Index := 0;
               if not HttpQueryInfo(hRequest, HTTP_QUERY_STATUS_CODE or HTTP_QUERY_FLAG_NUMBER,
                  @StatusCode, StatusSize, Index) then
                  raise Exception.Create('Não foi possível obter o status code da resposta.');

               // Lê a resposta
               repeat
                  ReadSize := 0;
                  if not InternetReadFile(hRequest, @Buffer, SizeOf(Buffer), ReadSize) then
                     raise Exception.Create('InternetReadFile falhou.');

                  if ReadSize = 0 then
                     Break;

                  Response.Write(Buffer, ReadSize);
               until False;

               if StatusCode <> 200 then
                  raise Exception.CreateFmt('Erro HTTP %d: %s', [StatusCode, Response.DataString]);

               Result := Response.DataString;
            finally
               InternetCloseHandle(hRequest);
            end;
         finally
            InternetCloseHandle(hConnect);
         end;
      finally
         InternetCloseHandle(hInet);
      end;
   finally
      Response.Free;
   end;
end;

end.

