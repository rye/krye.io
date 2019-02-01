require 'sinatra'

get '/' do
	erb :'index.html', locals: {for_hire: true}
end
