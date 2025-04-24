<lib name="MathLib" version="1.0">
  <description>
	This library provides basic mathematical operations.
  </description>
  <function name="add">
	<param name="a" type="int"/>
	<param name="b" type="int"/>
	<return type="int"/>
	<description>
	  This function adds two integers.
	</description>
	<body>
	  <return> a + b </return>
	</body>
  </function>

  <function name="subtract">
	<param name="a" type="int"/>
	<param name="b" type="int"/>
	<return type="int"/>
	<description>
	  This function subtracts two integers.
	</description>
	<body>
	  <return> a - b </return>
	</body>
  </function>

  <function name="multiply">
	<param name="a" type="int"/>
	<param name="b" type="int"/>
	<return type="int"/>
	<description>
	  This function multiplies two integers.
	</description>
	<body>
	  <return> a * b </return>
	</body>
  </function>

  <function name="divide">
	<param name="a" type="int"/>
	<param name="b" type="int"/>
	<return type="int"/>
	<description>
	  This function divides two integers.
	</description>
	<body>
	  <if>
		<condition> b == 0 </condition>
		<then>
		  <return> "Error: Division by zero" </return>
		</then>
		<else>
		  <return> a / b </return>
		</else>
	  </if>
	</body>
  </function>

  <function name="modulus">
	<param name="a" type="int"/>
	<param name="b" type="int"/>
	<return type="int"/>
	<description>
	  This function calculates the modulus of two integers.
	</description>
	<body>
	  <if>
		<condition> b == 0 </condition>
		<then>
		  <return> "Error: Division by zero" </return>
		</then>
		<else>
		  <return> a % b </return>
		</else>
	  </if>
	</body>
  </function>

  <function name="power">
	<param name="base" type="int"/>
	<param name="exponent" type="int"/>
	<return type="int"/>
	<description>
	  This function calculates the power of a number.
	</description>
	<body>
	  <if>
		<condition> exponent == 0 </condition>
		<then>
		  <return> 1 </return>
		</then>
		<else>
		  <return> base * 
		  	<call function="power">
		  	  <param> base </param>
		  	  <param> exponent - 1 </param>
		  	</call>
		  </return>
		</else>
	  </if>
	</body>
  </function>

  <function name="factorial">
	<param name="n" type="int"/>
	<return type="int"/>
	<description>
	  This function calculates the factorial of a number.
	</description>
	<body>
	  <if>
		<condition> n == 0 </condition>
		<then>
		  <return> 1 </return>
		</then>
		<else>
		  <return> n * 
		  	<call function="factorial">
		  	  <param> n - 1 </param>
		  	</call>
		  </return>
		</else>
	  </if>
	</body>
  </function>

  <function name="fibonacci">
	<param name="n" type="int"/>
	<return type="int"/>
	<description>
	  This function calculates the nth Fibonacci number.
	</description>
	<body>
	  <if>
		<condition> n == 0 </condition>
		<then>
		  <return> 0 </return>
		</then>
		<else>
		  <if>
			<condition> n == 1 </condition>
			<then>
			  <return> 1 </return>
			</then>
			<else>
			  <return>
			  	<call function="fibonacci">
			  	  <param> n - 1 </param>
			  	</call>
				+
			  	<call function="fibonacci">
			  	  <param> n - 2 </param>
			  	</call>
			  </return>
			</else>
		  </if>
		</else>
	  </if>
	</body>
  </function>
</lib>
