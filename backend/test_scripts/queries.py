import requests

def query_graph():
  q = '''
  {
    swaps(where:{pool_in:["0x88e6a0c2ddd26feeb64f039a2c41296fcb3f5640",
                          "0x8ad599c3a0ff1de082011efddc58f1908eb6e6d8",
                          "0xe0554a476a092703abdb3ef35c80e0d76d32939f"
                          "0x7BeA39867e4169DBe237d55C8242a8f2fcDcc387"]
                 timestamp_gt:1733080500 timestamp_lt:1733080560 }
          orderBy:timestamp orderDirection:desc )
    {
      amount0
      amount1
    }
  }'''
  url = 'https://gateway.thegraph.com/api/da162f6f59fe4400bb44cfb2f36d1336/subgraphs/id/5zvR82QoaXYFyDEKLZ9t6v9adgnptxYpKpSbxtgVENFV'
  header = {"Content-Type": "application/json"}
  
  r = requests.post( url, headers=header, json={"query":q})

  print( r.status_code )
  return r.json()["data"] 

def query_binance():
    url = "https://api.binance.com/api/v3/aggTrades?symbol=ETHUSDC&startTime=1733240400000&endTime=1733240460000"
    
    r = requests.get( url )
    print( r.status_code )
    return r.json()

